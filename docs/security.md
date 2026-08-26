# Dependency and scripting security

Dependency policy is defined in `deny.toml`. Run `scripts/security.sh` or `scripts/security.ps1` after dependency changes. The checks cover RustSec advisories, yanked crates, accepted licenses, duplicate-version visibility, and registry/git source policy.

Rhai is treated as a mod boundary, not trusted engine code. Runtime operation/call/collection budgets are defined by `RhaiSandboxLimits`; raising them should be treated as a resource-policy change and accompanied by adversarial tests.

Security workflow execution still depends on repository/account GitHub Actions availability; a runner-allocation failure is distinct from a passing or failing audit.
