#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        let mut engine = rhai::Engine::new();
        engine
            .set_max_expr_depths(256, 512)
            .set_max_string_size(1024 * 1024)
            .set_max_array_size(100_000)
            .set_max_map_size(20_000);
        let _ = engine.compile(source);
    }
});
