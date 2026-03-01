# nu_plugin_nw_ulid Commit Guidelines

This project follows conventional commit format with specific requirements.

## Severity Levels

| Severity | Sections                                                                |
|----------|-------------------------------------------------------------------------|
| error    | Commit Format, Types, Scopes, Subject Line, Accuracy, Breaking Changes |
| warning  | Body Guidelines                                                         |
| info     | Subject Line Style                                                      |

## Commit Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

Multiple scopes are allowed when a commit spans more than one area.
Separate scopes with a comma and no space:

```
<type>(<scope1>,<scope2>): <description>
```

## Types

Required. Must be one of:

| Type       | Use for                                               |
|------------|-------------------------------------------------------|
| `feat`     | New features or enhancements to existing features     |
| `fix`      | Bug fixes                                             |
| `docs`     | Documentation changes only                            |
| `refactor` | Code refactoring without behavior changes             |
| `chore`    | Maintenance tasks, dependency updates, config changes |
| `test`     | Test additions or modifications                       |
| `ci`       | CI/CD pipeline changes                                |
| `build`    | Build system or external dependency changes           |
| `perf`     | Performance improvements                              |
| `style`    | Code style changes (formatting, whitespace)           |

## Scopes

Required. Use scopes defined in `.omni-dev/scopes.yaml`:

- `cargo` - Cargo build configuration and manifests
- `ci` - CI/CD workflows and configuration
- `cli` - CLI command methods and argument handling
- `commands` - Individual command implementations
- `config` - Project configuration files and tooling setup
- `core` - Core types, constants, and engine internals
- `deps` - Dependency updates
- `docs` - Documentation
- `engine` - Core UlidEngine operations
- `error` - Error types and conversion
- `lib` - Library-level utilities and shared code
- `plugin` - Plugin registration, command dispatch
- `release` - Version bumps, release process
- `security` - Security warnings, rating system
- `style` - Style guide rules and conventions
- `test` - Test infrastructure and test utilities

For multi-scope commits, the scopes are correct when each listed scope
matches at least one modified file. Do not flag scopes as incorrect
when the commit legitimately spans multiple areas.

## Subject Line

- Keep under 72 characters total
- Use imperative mood: "add feature" not "added feature" or "adds feature"
- Be specific: avoid vague terms like "update", "fix stuff", "changes"

## Subject Line Style

- Use lowercase for the description
- No period at the end

## Accuracy

The commit message must accurately reflect the actual code changes:

- **Type must match changes**: Don't use `feat` for a bug fix, or `fix` for new functionality
- **Scope must match files**: The scope should reflect which area of code was modified
- **Description must be truthful**: Don't claim changes that weren't made
- **Mention significant changes**: If you add error handling, logging, or change behavior, mention it

Only flag accuracy errors when the commit message is clearly and
materially wrong. Do not flag minor terminology differences,
language-specific semantic debates, or cases where the description
is substantially correct even if slightly imprecise. Before reporting
an issue, verify your reasoning is internally consistent — if your
own explanation concludes the commit is actually correct, do not
report it.

## Body Guidelines

For significant changes (>50 lines or architectural changes), include a body:

- Explain what was changed and why
- Describe the approach taken
- Note any breaking changes or migration requirements
- Use bullet points for multiple related changes
- Reference issues in footer: `Closes #123` or `Fixes #456`

## Breaking Changes

For breaking changes:
- Add `!` after type/scope: `feat(cli)!: change output format`
- Include `BREAKING CHANGE:` footer with migration instructions

## Examples

### Simple change
```
fix(engine): handle invalid timestamp in ULID parsing
```

### Feature with body
```
feat(commands): add --json output to inspect command

Adds structured JSON output option to the inspect command for
programmatic consumption of ULID component data.

- Add --json flag to inspect command
- Implement JSON serialization for UlidComponents
- Update help text with JSON output examples

Closes #42
```

### Documentation
```
docs(docs): add ADR for multi-algorithm hash support
```

### Style guide update
```
docs(style): add command domain grouping guidelines
```

### Multiple scopes
```
style(engine,commands): standardise doc comments
```

```
refactor(engine,commands): extract shared validation logic

Moves ULID string validation into a shared helper used by both
the engine and individual command implementations.

- Extract validate_ulid_string() to engine module
- Update commands to use shared validator
- Remove duplicate validation logic
```

### Breaking change
```
feat(plugin)!: change command namespace from nw-ulid to ulid

BREAKING CHANGE: All commands are now under the `ulid` namespace
instead of `nw-ulid`. Update your scripts to use the new names.
```
