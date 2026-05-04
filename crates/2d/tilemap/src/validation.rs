use std::collections::{BTreeMap, BTreeSet};

use crate::model::{TileMap2d, TileRuleSet2d};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileWorldDiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileWorldDiagnostic {
    pub severity: TileWorldDiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct TileWorldValidationContext {
    pub tileset_exists: Option<bool>,
    pub ruleset_exists: Option<bool>,
    pub tile_count: Option<u32>,
    pub ruleset_tile_size: Option<(u32, u32)>,
}

pub fn validate_tile_world_contract(
    tilemap: &TileMap2d,
    ruleset: Option<&TileRuleSet2d>,
) -> Vec<TileWorldDiagnostic> {
    validate_tile_world_contract_with_context(tilemap, ruleset, &TileWorldValidationContext::default())
}

pub fn validate_tile_world_contract_with_context(
    tilemap: &TileMap2d,
    ruleset: Option<&TileRuleSet2d>,
    context: &TileWorldValidationContext,
) -> Vec<TileWorldDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_grid_shape(tilemap, &mut diagnostics);
    validate_asset_presence(tilemap, context, &mut diagnostics);

    if let Some(ruleset) = ruleset {
        validate_tile_size(tilemap, ruleset, context, &mut diagnostics);
        validate_symbols(tilemap, ruleset, &mut diagnostics);
        validate_markers(tilemap, ruleset, &mut diagnostics);
        validate_ruleset(ruleset, context, &mut diagnostics);
    }

    diagnostics
}

fn validate_grid_shape(tilemap: &TileMap2d, diagnostics: &mut Vec<TileWorldDiagnostic>) {
    if tilemap.grid.is_empty() {
        diagnostics.push(TileWorldDiagnostic {
            severity: TileWorldDiagnosticSeverity::Warning,
            message: "TileMap2D grid is empty".to_owned(),
        });
        return;
    }

    let expected_width = tilemap.grid[0].chars().count();
    for (row_index, row) in tilemap.grid.iter().enumerate() {
        let width = row.chars().count();
        if width != expected_width {
            diagnostics.push(TileWorldDiagnostic {
                severity: TileWorldDiagnosticSeverity::Error,
                message: format!(
                    "TileMap2D grid row {row_index} has width {width}, expected {expected_width}"
                ),
            });
        }
    }
}

fn validate_symbols(
    tilemap: &TileMap2d,
    ruleset: &TileRuleSet2d,
    diagnostics: &mut Vec<TileWorldDiagnostic>,
) {
    let terrain_symbols = ruleset
        .terrains
        .iter()
        .map(|terrain| terrain.symbol)
        .collect::<BTreeSet<_>>();
    let marker_symbols = ruleset
        .markers
        .iter()
        .map(|marker| marker.symbol)
        .collect::<BTreeSet<_>>();
    let mut unknown = BTreeSet::new();
    let empty = ruleset.empty_symbol();

    for row in &tilemap.grid {
        for symbol in row.chars() {
            if symbol == empty || terrain_symbols.contains(&symbol) || marker_symbols.contains(&symbol) {
                continue;
            }
            unknown.insert(symbol);
        }
    }

    for symbol in unknown {
        diagnostics.push(TileWorldDiagnostic {
            severity: TileWorldDiagnosticSeverity::Warning,
                message: format!("TileMap2D grid uses symbol '{symbol}' that is not declared in ruleset"),
        });
    }
}

fn validate_markers(
    tilemap: &TileMap2d,
    ruleset: &TileRuleSet2d,
    diagnostics: &mut Vec<TileWorldDiagnostic>,
) {
    for marker in &ruleset.markers {
        let count = tilemap
            .grid
            .iter()
            .flat_map(|row| row.chars())
            .filter(|symbol| *symbol == marker.symbol)
            .count();

        if let Some(max_count) = marker.max_count {
            if count > max_count {
                diagnostics.push(TileWorldDiagnostic {
                    severity: TileWorldDiagnosticSeverity::Error,
                    message: format!(
                        "Marker '{}' appears {count} times, max_count is {max_count}",
                        marker.name
                    ),
                });
            }
        }
    }
}

fn validate_ruleset(
    ruleset: &TileRuleSet2d,
    context: &TileWorldValidationContext,
    diagnostics: &mut Vec<TileWorldDiagnostic>,
) {
    let mut seen = BTreeMap::<char, &str>::new();
    for terrain in &ruleset.terrains {
        if terrain.symbol == '\0' {
            diagnostics.push(TileWorldDiagnostic {
                severity: TileWorldDiagnosticSeverity::Error,
                message: format!("Terrain '{}' has no symbol", terrain.name),
            });
            continue;
        }

        if let Some(value) = &terrain.unknown_collision {
            diagnostics.push(TileWorldDiagnostic {
                severity: TileWorldDiagnosticSeverity::Error,
                message: format!(
                    "Terrain '{}' uses unknown collision '{}'",
                    terrain.name, value
                ),
            });
        }

        if let Some(previous) = seen.insert(terrain.symbol, terrain.name.as_str()) {
            diagnostics.push(TileWorldDiagnostic {
                severity: TileWorldDiagnosticSeverity::Error,
                message: format!(
                    "Terrain '{}' reuses symbol '{}' already used by '{}'",
                    terrain.name, terrain.symbol, previous
                ),
            });
        }

        if terrain.variants.tile_id_for(crate::TileVariantKind2d::Single).is_none() {
            diagnostics.push(TileWorldDiagnostic {
                severity: TileWorldDiagnosticSeverity::Warning,
                message: format!("Terrain '{}' has no usable tile variant", terrain.name),
            });
        }

        if let Some(tile_count) = context.tile_count {
            for tile_id in terrain.variants.iter_tile_ids() {
                if tile_id >= tile_count {
                    diagnostics.push(TileWorldDiagnostic {
                        severity: TileWorldDiagnosticSeverity::Error,
                        message: format!(
                            "Terrain '{}' references tile id {tile_id}, but tileset tile_count is {tile_count}",
                            terrain.name
                        ),
                    });
                }
            }
        }
    }

    for marker in &ruleset.markers {
        if marker.symbol == '\0' {
            diagnostics.push(TileWorldDiagnostic {
                severity: TileWorldDiagnosticSeverity::Error,
                message: format!("Marker '{}' has no symbol", marker.name),
            });
        }
        if let Some(previous) = seen.insert(marker.symbol, marker.name.as_str()) {
            diagnostics.push(TileWorldDiagnostic {
                severity: TileWorldDiagnosticSeverity::Error,
                message: format!(
                    "Marker '{}' reuses symbol '{}' already used by '{}'",
                    marker.name, marker.symbol, previous
                ),
            });
        }
    }
}

fn validate_asset_presence(
    tilemap: &TileMap2d,
    context: &TileWorldValidationContext,
    diagnostics: &mut Vec<TileWorldDiagnostic>,
) {
    if tilemap.tileset.as_str().trim().is_empty() || context.tileset_exists == Some(false) {
        diagnostics.push(TileWorldDiagnostic {
            severity: TileWorldDiagnosticSeverity::Error,
            message: format!("Tileset '{}' is missing", tilemap.tileset.as_str()),
        });
    }
    if tilemap.ruleset.is_some() && context.ruleset_exists == Some(false) {
        diagnostics.push(TileWorldDiagnostic {
            severity: TileWorldDiagnosticSeverity::Error,
            message: format!(
                "Ruleset '{}' is missing",
                tilemap.ruleset.as_ref().map(|key| key.as_str()).unwrap_or("")
            ),
        });
    }
}

fn validate_tile_size(
    tilemap: &TileMap2d,
    ruleset: &TileRuleSet2d,
    context: &TileWorldValidationContext,
    diagnostics: &mut Vec<TileWorldDiagnostic>,
) {
    let expected = ruleset.tile_size.or(context.ruleset_tile_size);
    if let Some((x, y)) = expected {
        if (tilemap.tile_size.x - x as f32).abs() > f32::EPSILON
            || (tilemap.tile_size.y - y as f32).abs() > f32::EPSILON
        {
            diagnostics.push(TileWorldDiagnostic {
                severity: TileWorldDiagnosticSeverity::Warning,
                message: format!(
                    "TileMap2D tile_size is {}x{}, ruleset expects {}x{}",
                    tilemap.tile_size.x, tilemap.tile_size.y, x, y
                ),
            });
        }
    }
}
