# Domain Waterfall

Every domain plugin follows the same waterfall.

```txt
Source
-> Roles / Capabilities
-> Contribution
-> Response
-> Coverage
-> Candidate
-> Target
-> Consumer
-> Final Output
-> Diagnostics
-> Tests
```

## Definitions

Source:
The object, system, or plugin that owns initial data.

Contribution:
A semantic declaration of influence on a domain.

Response:
How strongly the domain should react.

Coverage:
Where and how the contribution applies.

Candidate:
Resolved, validated domain work item.

Target:
Named buffer, field, graph node, or semantic output.

Consumer:
System or pass that consumes targets.

Diagnostics:
Trace from source to output.

Tests:
Proof that the waterfall works.

## Hard rule

Contribution is not an effect.

Example:

```txt
Light2D does not execute lens flare.
Light2D emits CameraOpticsContribution2d.
CameraOptics resolves candidates.
CameraOptics writes SceneHighlight / SceneEmissive.
CameraOptics consumer produces lens artifacts.
```
