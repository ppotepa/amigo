# Render Composition

The render API exposes:

- `RenderSpace`
- `RenderLayerId`
- `CameraBinding`
- `CompositionLayer`

The target model is:

- `World3D`
- `World2D`
- `Screen2D`
- `Ui`
- `Gizmos`
- `DebugOverlay`
- `Present`

Legacy pass names such as `World`, `GameUi`, and `DebugOverlay` may still exist during migration, but composition should progressively move toward layer-based render planning.
