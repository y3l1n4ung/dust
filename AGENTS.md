# Dust Agent Instructions

## Scope
These instructions apply to the entire Dust repository.

## Working Model
- Treat this repository as an open-source project managed through GitHub.
- Use `gh` for pull requests, GitHub Actions checks, issues, labels, milestones, and release triage when network access is available.
- Keep work scoped to the active branch and never rewrite unrelated user changes.
- Prefer small, reviewable commits with a clear reason and validation evidence.

## Project Management
- Before starting large work, check the current branch, PR, failing checks, and open issue context when available.
- Track non-trivial follow-up work as GitHub issues instead of leaving TODO-only notes in code.
- Use labels consistently: `bug`, `security`, `performance`, `docs`, `testing`, `good first issue`, `breaking-change`, and `needs-design`.
- Keep PR descriptions concrete: problem, solution, tests, risk, and follow-up issues.
- Do not merge or mark work complete while required CI checks are failing.

## Open-Source Security
- Never commit secrets, tokens, API keys, private URLs, or machine-local credentials.
- Use environment variables or GitHub Actions secrets for sensitive values.
- Review workflow changes for least privilege; keep `GITHUB_TOKEN` permissions minimal.
- Do not add network-dependent tests unless explicitly required and isolated.
- Treat generated code, install scripts, and release scripts as security-sensitive surfaces.
- For dependency changes, check license fit, maintenance status, transitive risk, and lockfile impact.
- If a vulnerability is suspected, create a private/security issue or advisory path first; do not publish exploit details in public issues.

## Validation Rules
- Use `flutter` commands for Flutter packages and examples.
- Use `dart` commands for Dart-only packages.
- Use `cargo` commands for Rust crates.
- Prefer the repo scripts when checking broad changes: `./scripts/lint.sh` and `./scripts/test.sh`.
- Use `gh pr checks` and GitHub Actions logs to verify CI after every push that affects workflows, build logic, tests, or generated outputs.

## Codegen Standards
- Keep generated fixture snapshots analyzer-safe. Dart generated snapshots used only as test fixtures should use `.dart.snapshot`, not `.dart` or `.g.dart`.
- Generated real outputs in examples remain `.g.dart`.
- Prefer shared emitters/templates over ad hoc string assembly.
- Prefer parser/IR facts over plugin-local source parsing.
- Keep tests exact and deterministic; use full expected output snapshots where practical.

## Release Discipline
- Public API stability matters. If an API is marked stable, do not change it without a documented migration plan.
- Stable surfaces currently include data classes, JSON, validation, and HTTP client APIs.
- Route, database, and state-management APIs are still hardening; document risks clearly in PRs.
- Update docs and examples together with public behavior changes.
