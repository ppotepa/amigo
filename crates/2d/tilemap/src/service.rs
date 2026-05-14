use std::sync::Mutex;

use crate::model::{TileMap2dDrawCommand, TileRuleSet2d};
use crate::resolver::resolve_tilemap;
use amigo_assets::AssetKey;

#[derive(Debug, Default)]
pub struct TileMap2dSceneService {
    commands: Mutex<Vec<TileMap2dDrawCommand>>,
}

impl TileMap2dSceneService {
    pub fn queue(&self, command: TileMap2dDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("tilemap2d scene service mutex should not be poisoned");
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("tilemap2d scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<TileMap2dDrawCommand> {
        self.commands
            .lock()
            .expect("tilemap2d scene service mutex should not be poisoned")
            .clone()
    }

    pub fn sync_ruleset_for_asset(
        &self,
        ruleset_asset: &AssetKey,
        ruleset: &TileRuleSet2d,
    ) -> usize {
        let mut commands = self
            .commands
            .lock()
            .expect("tilemap2d scene service mutex should not be poisoned");
        let mut updated = 0;

        for command in commands.iter_mut() {
            if command.tilemap.ruleset.as_ref() != Some(ruleset_asset) {
                continue;
            }
            command.tilemap.resolved = Some(resolve_tilemap(&command.tilemap, ruleset));
            updated += 1;
        }

        updated
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}
