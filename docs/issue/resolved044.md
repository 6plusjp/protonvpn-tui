# resolved044 - Refactor connect_country and connect_city to use connect_with_args

## Summary

Refactored `connect_country` and `connect_city` to use `connect_with_args`.

## Implementation

Extended `connect_with_args` to accept optional arguments:

```rust
fn connect_with_args(&self, flag: &str, fallback_name: &str, args: &[&str]) -> AppResult<ConnectResult>
```

### Changes

- `connect_with_args`: Added `args: &[&str]` parameter
- `connect_country`: Now delegates to `connect_with_args("--country", target, &[target])`
- `connect_city`: Now delegates to `connect_with_args("--city", city_arg, &[city_arg])`
- All existing callers updated to pass empty slice `&[]`

## Result

All connect functions now use `connect_with_args`:

- `connect_fastest()` - ✓ uses `connect_with_args`
- `connect_random()` - ✓ uses `connect_with_args`
- `connect_p2p()` - ✓ uses `connect_with_args`
- `connect_tor()` - ✓ uses `connect_with_args`
- `connect_securecore()` - ✓ uses `connect_with_args`
- `connect_country(target)` - ✓ uses `connect_with_args`
- `connect_city(city_arg)` - ✓ uses `connect_with_args`

## Related

- resolved040 - Add keyboard shortcuts
