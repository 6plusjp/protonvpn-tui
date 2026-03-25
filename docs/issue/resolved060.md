# issue060: Documentation - src/ui/AGENTS.md references non-existent file

## Summary

The file `src/ui/AGENTS.md` contains outdated documentation that references `components/styles.rs` which does not exist.

## Problem

```markdown
## Module Structure

```
src/ui/
├── components/     # Reusable widgets
│   ├── styles.rs   # ← THIS FILE DOES NOT EXIST
```

## Actual Structure

All theme/styling code is in:

- `src/ui/styles.rs` - Theme and ThemeMode definitions

There is no `src/ui/components/styles.rs`.

## Files Affected

- `src/ui/AGENTS.md` - outdated documentation

## Recommendation

1. Remove the reference to `components/styles.rs` from `src/ui/AGENTS.md`
2. Or remove `src/ui/AGENTS.md` entirely (see issue: misplaced AGENTS.md files)

## Severity

🟡 MEDIUM - Documentation inaccuracy

## Labels

`documentation`
