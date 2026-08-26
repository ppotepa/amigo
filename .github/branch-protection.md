# Main branch ruleset

The canonical intended ruleset is stored at `.github/rulesets/main.json` so repository policy is reviewable beside source.

It targets `main`, blocks deletion and non-fast-forward updates, and requires the stable validation checks once GitHub-hosted Actions execution is available.

The connected GitHub API available to repository automation is read-only for rulesets/branch-protection settings, so committing this file does **not** activate the server-side rule. Import/apply the ruleset in GitHub repository settings (or through an authorized administration API) and verify that `main` reports protected before treating policy as enforced.
