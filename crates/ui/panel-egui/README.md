# External egui panel host

Native consumer of `amigo-panel-api`, launched by the engine over private pipes.
This crate owns widgets and presentation only. It does not import NPR or directly
mutate world state. Requires Rust 1.92+ and eframe 0.35 (WGPU 29). The public app
entrypoint dispatches its internal `--runtime-panel-client` mode here.
