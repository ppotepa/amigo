use amigo_relight_2d_plugin::api::Relight2dContribution;

use crate::api::BeaconLight2dSource;

pub fn beacon_to_relight_contribution(beacon: &BeaconLight2dSource) -> Relight2dContribution {
    Relight2dContribution {
        source_id: beacon.id.clone(),
        intensity: beacon.intensity,
        color_rgba: beacon.color_rgba,
    }
}
