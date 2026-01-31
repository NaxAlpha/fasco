# Fasco Coding Principles

## Structure
- **Flat over deep**: Prefer single-file features over complex architecture
- **File size limit**: 500-1000 lines max; split if exceeded
- **Tests near code**: Keep test files close to implementation

## Testing
- **Test-first**: Write tests before wiring code
- **Coverage target**: >80% required
- **All code testable**: Design for testability from start

## Quality Gates
- **Run linter**: `cargo clippy -- -D warnings`
- **Format**: `cargo fmt`
- **Verify tests**: Always run before commit

## Code Style
- **Explicit names**: `calculateMonthlyRevenue()` not `calc()`
- **Minimal comments**: Let names/types document intent
- **Clear imports**: Explicit imports/exports for discoverability
- **Compact logs**: Cover everything, make debugging easy

## Principles
- **Fail fast**: Verify everything at boundaries
- **Atomic changes**: Small, inspectable steps
- **Observability by default**: Log what matters
- **Design for change**: Not certainty
- **Make quality executable**: Automated checks

## README.md Maintenance
- **Compact overview**: AI should understand project without reading files
- **Keep proportions**: Small changes = small updates; don't over-highlight
- **Include**: Summary, architecture (1-line per file), tech stack, engineering preferences
- **Include**: Code style guide, lessons learned, external docs links (concise)
- **Exclude**: Deep implementation details (requires file exploration)
- **Purpose**: Quick onboarding; code changes require deeper exploration
