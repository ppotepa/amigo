# Main branch protection

The repository CI check is named `ci / validate` and is intended to be required for `main`.

Recommended repository rule:

- require `ci / validate` before updates to `main`;
- require the branch to be up to date before merge when pull requests are used;
- block force pushes and deletions;
- allow administrators to bypass only for incident recovery.

This file documents the expected repository setting so it can be audited alongside source. GitHub branch protection itself is a repository setting, not a source-controlled file.
