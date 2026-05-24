mod anim_clip;
mod app;
mod assets;
mod export;
mod math;
mod mesh;
mod pipeline;
mod renderer;
mod selftest;
mod settings;
mod state;
mod ui;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let mut args = std::env::args_os().skip(1);
    if let Some(arg) = args.next()
        && arg == "--self-test"
    {
        return selftest::run(args.next().map(std::path::PathBuf::from));
    }
    app::run()
}
