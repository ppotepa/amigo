import "mod:tar_fx/ids" as ids;
import "mod:tar_fx/math" as math;

fn reset(world_api) {
    world_api.state.set_float("rain.next_gust", world_api.random.range(3.5, 8.5));
    world_api.state.set_float("rain.gust_start", -100.0);
    world_api.state.set_float("rain.gust_until", -100.0);
    world_api.state.set_float("rain.gust_duration", 1.0);
    world_api.state.set_float("rain.gust_strength", 0.0);
    world_api.state.set_float("rain.gust_dir", 1.0);

    world_api.particles.set_intensity(ids::rain_far_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_mid_entity(), 0.0);
    world_api.particles.set_intensity(ids::rain_near_entity(), 0.0);
    world_api.particles.set_wind(ids::rain_far_entity(), 68.0, -320.0, 0.12);
    world_api.particles.set_wind(ids::rain_mid_entity(), 112.0, -500.0, 0.16);
    world_api.particles.set_wind(ids::rain_near_entity(), 150.0, -680.0, 0.18);
}


fn update(world_api, time, far_light, mid_light, near_light) {
    if time >= world_api.state.get_float("rain.next_gust") {
        let duration = world_api.random.range(0.8, 2.4);
        let strength = world_api.random.range(0.35, 1.0);
        let direction = if world_api.random.range(0.0, 1.0) < 0.18 {
            -1.0
        } else {
            1.0
        };

        world_api.state.set_float("rain.gust_start", time);
        world_api.state.set_float("rain.gust_until", time + duration);
        world_api.state.set_float("rain.gust_duration", duration);
        world_api.state.set_float("rain.gust_strength", strength);
        world_api.state.set_float("rain.gust_dir", direction);
        world_api.state.set_float("rain.next_gust", time + duration + world_api.random.range(4.0, 10.0));
    }

    let gust = if time < world_api.state.get_float("rain.gust_until") {
        let phase = math::clamp01((time - world_api.state.get_float("rain.gust_start")) / world_api.state.get_float("rain.gust_duration"));
        let envelope = (phase * 3.14159).sin();
        envelope * world_api.state.get_float("rain.gust_strength")
    } else {
        0.0
    };
    let dir = world_api.state.get_float("rain.gust_dir");

    world_api.particles.set_intensity(ids::rain_far_entity(), math::clamp01(far_light * 0.62 + gust * 0.16));
    world_api.particles.set_intensity(ids::rain_mid_entity(), math::clamp01(mid_light * 0.52 + gust * 0.14));
    world_api.particles.set_intensity(ids::rain_near_entity(), math::clamp01(near_light * 0.42 + gust * 0.12));
    world_api.particles.set_wind(
        ids::rain_far_entity(),
        68.0 + dir * gust * 260.0,
        -320.0 - gust * 54.0,
        0.12 + gust * 0.22
    );
    world_api.particles.set_wind(
        ids::rain_mid_entity(),
        112.0 + dir * gust * 430.0,
        -500.0 - gust * 92.0,
        0.16 + gust * 0.27
    );
    world_api.particles.set_wind(
        ids::rain_near_entity(),
        150.0 + dir * gust * 620.0,
        -680.0 - gust * 130.0,
        0.18 + gust * 0.32
    );
}


