fn drive_ship(world, entity, thrust_action, turn_action) {
    world.arcade.drive_freeflight(entity, thrust_action, turn_action)
}

fn drive_ship_with_thruster(world, entity, thruster, thrust_action, turn_action) {
    world.arcade.drive_freeflight_with_thruster(entity, thruster, thrust_action, turn_action)
}
