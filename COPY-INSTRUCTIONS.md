# Copy instructions

This bundle should be applied selectively.

## Recommended repo operations

```text
MODIFY AGENTS.md
  Replace the existing root file with bundle/AGENTS.md.

ADD PROJECT.md
  Add bundle/PROJECT.md if the repo still has no PROJECT.md.

ADD docs/architecture/postfx-render-coupling-audit.md
  Copy audits/postfx-render-coupling-audit.md into docs/architecture/ if you want the Etap 1 audit in canonical docs.

ADD docs/agent-workflow.md or docs/architecture/agent-workflow.md
  Copy docs/01-agent-workflow.md if you want an in-repo workflow doc.
```

## Avoid as a single giant commit

Do not copy every file into the repo in one architecture refactor commit. Keep this bundle as an external documentation archive and promote files gradually.

## Validation for docs-only copy

```powershell
git status --short
git diff --check
```

## Validation after replacing root AGENTS.md / adding PROJECT.md

```powershell
git status --short
git diff -- AGENTS.md PROJECT.md
git diff --check
```
