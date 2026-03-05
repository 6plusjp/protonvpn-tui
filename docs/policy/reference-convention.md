# Document Reference Convention

## Purpose

Establishes a consistent way to mark documents that must be read or loaded
as prerequisite context, distinguishing them from paths mentioned as
examples or illustrations.

## Syntax

Use a markdown link with an `@` prefix to indicate **required context**:

    [@docs/policy/policy.md](docs/policy/policy.md)

A bare backtick path without `@` is informational or illustrative only:

    `docs/issue/issue003.md`

## Rules

1. An `@`-reference means "this file MUST be loaded before proceeding."
2. Use `@`-references in CLAUDE.md, policy files, issue documents, and
   skill definitions where prerequisite files exist.
3. `@`-references use project-root-relative paths (no leading slash).
4. Do not `@`-reference files in `docs/reference/`.
5. When an `@`-referenced file itself contains `@`-references, load them
   recursively (up to 2 levels deep).
6. The markdown link format `[@path](path)` ensures clickable navigation
   in GitHub and IDEs.

## Revision History

- 2026-03-05: Initial version
