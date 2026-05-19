# Contributions and Candidates

Contribution means semantic participation in a domain.

Candidate means resolved domain work.

## Status values

```txt
Active
Inactive
NotDeclared
Unsupported
Noop
Error
```

## Rules

* Missing contribution creates no candidate.
* Unsupported contribution cannot become an active candidate.
* Renderer cannot synthesize contribution from luma, brightness, or object type.
* Auto-default contribution may only be created during hydration or validation.
* Candidate diagnostics must explain source, domain, status, reason, and targets.
