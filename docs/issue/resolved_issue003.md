# issue003: Cities view bugs - navigation, cache, and parsing

## Summary

Cities view has three bugs:
1. j/k navigation doesn't work (can't move between cities) — **FIXED**
2. Cities cache is not loaded when returning to Servers view — **FIXED**
3. Cities not parsed correctly from `protonvpn cities` output

---

## Bug 1 & 2: Navigation and Cache (FIXED)

[See original issue description above - these have been fixed]

---

## Bug 3: City Parsing Failure

### Symptom

Running `protonvpn cities --country JP` produces:
```
Cities in Japan:
City    Features
------  ----------------
Osaka   P2P
Tokyo   P2P, Secure Core 
```

But in the UI, cities are displayed as "Cities, City, ------, Osaka, Tokyo" instead of just "Osaka, Tokyo".

### Root Cause

The parsing function in `src/vpn/client.rs:359-380` has two issues:

1. **Doesn't skip header lines**: Lines like "Cities in Japan:", "City    Features", "------  ----------------" are treated as city data
2. **Incorrect feature parsing**: Uses `split_whitespace()` which splits "P2P, Secure Core" into `["P2P,", "Secure", "Core"]` instead of keeping it as one feature

```rust
// Current broken implementation
pub(crate) fn parse_cities_with_features(&self, output: &str) -> Vec<City> {
    let mut cities = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0].to_string();
        let features: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        cities.push(City::with_features(name, features));
    }

    cities
}
```

### Fix Required

```rust
pub(crate) fn parse_cities_with_features(&self, output: &str) -> Vec<City> {
    let mut cities = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        
        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Skip header lines - lines that don't start with a letter
        let first_char = line.chars().next();
        if !first_char.map_or(false, |c| c.is_alphabetic()) {
            continue;
        }

        // Split only on tabs or multiple spaces (not inside feature strings)
        // Format: "CityName    Feature1, Feature2"
        let parts: Vec<&str> = line.splitn(2, |c| c.is_whitespace()).collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0].to_string();
        
        // Features are everything after the city name, join back together
        let features = if parts.len() > 1 && !parts[1].is_empty() {
            vec![parts[1].to_string()]  // Keep as single string: "P2P, Secure Core"
        } else {
            Vec::new()
        };

        cities.push(City::with_features(name, features));
    }

    cities
}
```

Key changes:
1. Skip lines that don't start with a letter (skips "Cities in Japan:", "------", etc.)
2. Use `splitn(2, ...)` to split only on the first whitespace occurrence - keeps feature string intact

### Alternative Fix (simpler)

If features are comma-separated and we want individual feature tags:

```rust
pub(crate) fn parse_cities_with_features(&self, output: &str) -> Vec<City> {
    let mut cities = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        
        if line.is_empty() {
            continue;
        }

        // Skip header lines
        let first_char = line.chars().next().unwrap_or('\0');
        if !first_char.is_alphabetic() {
            continue;
        }

        // Find first whitespace position to split name and features
        if let Some(pos) = line.find(|c: char| c.is_whitespace()) {
            let name = line[..pos].to_string();
            let features_str = line[pos..].trim();
            
            // Split features by comma, trim each
            let features: Vec<String> = features_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            cities.push(City::with_features(name, features));
        } else {
            // No features, just city name
            cities.push(City::new(line.to_string()));
        }
    }

    cities
}
```

This splits features by comma: "P2P, Secure Core" → ["P2P", "Secure Core"]

---

## Acceptance Criteria

- [x] j/k keys navigate correctly in Cities view (move through cities list)
- [x] Ctrl+d (page down) works in Cities view
- [x] Ctrl+u (page up) works in Cities view
- [x] g/G (first/last) navigation works in Cities view
- [x] Cities cached during Cities view are displayed in Servers view after returning
- [x] Cities are parsed correctly from `protonvpn cities` output (skip headers, parse features)
- [x] Server view shows correct cities for each country

---

## Fix Applied

### Bug 1: Navigation Fix (`src/state/app_state.rs`)

Added `get_selection_bounds()` method that returns `current_cities.len()` when in Cities view:

```rust
fn get_selection_bounds(&self) -> usize {
    match self.current_view {
        crate::state::AppView::Cities => self.current_cities.len(),
        _ => self.filtered_servers().len(),
    }
}
```

### Bug 2: Cache Fix (`src/vpn/client.rs`)

Load cities from cache in `countries_to_servers()`:

```rust
cities: self.cache.cities.get(code).cloned().unwrap_or_default(),
```

### Bug 3: Parsing Fix (`src/vpn/client.rs`)

Improved `parse_cities_with_features()` to skip headers and split features by comma:

```rust
pub(crate) fn parse_cities_with_features(&self, output: &str) -> Vec<City> {
    let mut cities = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Skip lines that don't start with a letter (e.g., "------")
        if !line.starts_with(|c: char| c.is_alphabetic()) {
            continue;
        }

        // Split name (before first whitespace) from features (after)
        let name_end = line.find(|c: char| c.is_whitespace());
        let (name, features_str) = match name_end {
            Some(pos) => (line[..pos].to_string(), line[pos..].trim()),
            None => (line.to_string(), ""),
        };

        // Skip header lines: "Cities in ...", "City Features"
        if name == "Cities" || features_str == "Features" {
            continue;
        }

        // Features are comma-separated: "P2P, Secure Core" → ["P2P", "Secure Core"]
        let features: Vec<String> = features_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        cities.push(City::with_features(name, features));
    }

    cities
}
```

---

## Commits

- `fa6ac8b` fix: use view-aware selection bounds for Cities navigation
- `a428315` fix: remove duplicate Ctrl+U key handler
- `f1afefa` fix: improve city parsing and load cities from cache
