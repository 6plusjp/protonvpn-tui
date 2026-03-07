# issue017: Split Large Functions - handle_key() and render()

## Status: OPEN

## Priority: Low

## Description

Split large functions in `src/ui/app.rs` to improve code readability and maintainability.

## Background

During issue016 code review, it was noted that `handle_key()` and `render()` functions in `src/ui/app.rs` are too large:
- `handle_key()` - 270+ lines
- `render()` - 60+ lines

This was marked as "Low Priority" / "backlog" in issue016 due to the large refactoring effort required.

## Tasks

### handle_key() - 270+ lines

Current structure handles many key events directly. Consider:
- Extract key handlers by view type (servers, stats, settings, logs, help)
- Group related key handling logic into separate private methods
- Use pattern matching to dispatch to specialized handlers

### render() - 60+ lines

Current structure renders different views. Consider:
- Extract view-specific rendering to separate methods
- Move common rendering logic to shared utilities
- Consider using a render trait or strategy pattern

## Affected Files

- `src/ui/app.rs`

## Related Issues

- resolved_issue016: Code Review - Potential Improvements (Category 6 - Code Quality)

## Tags

- refactoring
- code-quality
- low-priority
