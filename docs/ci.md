# CI execution and runner diagnostics

GitHub-hosted jobs can fail before checkout when Actions is disabled, runner allocation is unavailable, or account/repository policy blocks execution. A pre-run failure is identifiable by a job with no steps and no runner id.

Repository workflows intentionally use `ubuntu-latest` and require only `contents: read`. If a run has `steps: []`, resolve it in repository/account Actions settings; code changes cannot allocate a hosted runner.

For environments where hosted runners are unavailable, run `scripts/ci-local.sh` (or `scripts/ci-local.ps1`) from a recursive clone. It executes the same validation entry point as CI.
