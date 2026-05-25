# Scopes Pipeline

Scopes provides diagnostics for analyzing rendered image targets.

## Flow
- Scene data declares a `Scopes` tooling component.
- Runtime resolves a scope target plan.
- The backend reads the final composite or other declared analysis target.
- Scope measurements are written into `DiagnosticsSnapshot`.

## Targets
- Reads `FinalComposite` in the public diagnostics target plan.
- May inspect scene color, highlight, or emissive targets when declared by the
  active scope.
- Writes `DiagnosticsSnapshot`.
