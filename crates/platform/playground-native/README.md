# Playground native presentation

Windows-only companion transport, shared-memory mapping and DX12 presentation.
No NPR settings or camera policy live here. The authenticated named pipe binds
the parent and companion process IDs and uses a separate one-use stdin secret.
Messages contain metadata and duplicated handles; pixels never cross the pipe.

`NativeSurface` owns a child HWND and forwards CSS-relative input. `GpuPresenter`
opens textures and fences on the producer's adapter LUID, performs GPU copies and
signals consumption after use. `RgbaMapping` maps WebView2-owned buffers into the
engine process. JavaScript receives only read-only buffer views via WebView2.

Run `rtk cargo test -p amigo-playground-native`. The GPU/resize/retirement test is
owned by the exporter: `rtk cargo test -p amigo-render-wgpu shared_texture_roundtrip -- --ignored`.
It requires a Windows desktop and DX12 adapter. macOS/Linux native presenters are
not implemented.
