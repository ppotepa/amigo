use amigo_assets::{AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest};
use amigo_scripting_api::{ScriptEvent, ScriptEventQueue};

use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;

pub(crate) struct AssetsConsoleCommandHandler;

impl ConsoleCommandHandler for AssetsConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "assets-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![
            ConsoleCommandDescriptor {
                name: "assets",
                aliases: &[],
                category: "assets",
                help: "Show asset catalog summary.",
                usage: "assets",
                examples: &["assets"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "asset.reload",
                aliases: &[],
                category: "assets",
                help: "Reload an asset by key.",
                usage: "asset.reload <asset-key>",
                examples: &["asset.reload they-are-rotten/layered-images/neon-alley"],
                dev_only: true,
            },
        ]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "assets" || command.name == "asset" || command.name.starts_with("asset.")
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        mut command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        normalize_legacy_asset_command(&mut command);
        match command.name.as_str() {
            "assets" => assets_summary(ctx),
            "asset.reload" => {
                let Some(asset_key) = command.args.first() else {
                    return ConsoleCommandResult::error("usage: asset.reload <asset-key>");
                };
                let catalog = match ctx.required::<AssetCatalog>() {
                    Ok(catalog) => catalog,
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                let events = match ctx.required::<ScriptEventQueue>() {
                    Ok(events) => events,
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                let result =
                    request_asset_reload(catalog.as_ref(), asset_key, AssetLoadPriority::Immediate);
                if matches!(result, ConsoleCommandResult::Ok(_)) {
                    events.publish(ScriptEvent::new(
                        "dev-console.asset-reload-requested",
                        vec![asset_key.clone()],
                    ));
                }
                result
            }
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}

pub fn request_asset_reload(
    asset_catalog: &AssetCatalog,
    asset_key: &str,
    priority: AssetLoadPriority,
) -> ConsoleCommandResult {
    let asset_key = AssetKey::new(asset_key);
    if asset_catalog.manifest(&asset_key).is_none() {
        return ConsoleCommandResult::error(format!(
            "cannot reload unknown asset `{}`",
            asset_key.as_str()
        ));
    }
    asset_catalog.request_reload(AssetLoadRequest::new(asset_key.clone(), priority));
    ConsoleCommandResult::ok(format!("queued asset reload for `{}`", asset_key.as_str()))
}

fn assets_summary(ctx: &ConsoleCommandContext<'_>) -> ConsoleCommandResult {
    let catalog = match ctx.required::<AssetCatalog>() {
        Ok(catalog) => catalog,
        Err(error) => return ConsoleCommandResult::error(error.to_string()),
    };
    let loaded = catalog
        .loaded_assets()
        .into_iter()
        .map(|asset| asset.key.as_str().to_owned())
        .collect::<Vec<_>>();
    let prepared = catalog
        .prepared_assets()
        .into_iter()
        .map(|asset| format!("{} ({})", asset.key.as_str(), asset.kind.as_str()))
        .collect::<Vec<_>>();
    let failed = catalog
        .failed_assets()
        .into_iter()
        .map(|asset| format!("{}: {}", asset.key.as_str(), asset.reason))
        .collect::<Vec<_>>();
    let pending = catalog
        .pending_loads()
        .into_iter()
        .map(|request| request.key.as_str().to_owned())
        .collect::<Vec<_>>();

    ConsoleCommandResult::ok(format!(
        "assets loaded={} prepared={} failed={} pending={}",
        display_string_list(&loaded),
        display_string_list(&prepared),
        display_string_list(&failed),
        display_string_list(&pending)
    ))
}

fn normalize_legacy_asset_command(command: &mut ParsedConsoleCommand) {
    if command.name != "asset" {
        return;
    }
    let Some(verb) = command.args.first().cloned() else {
        return;
    };
    command.name = format!("asset.{verb}");
    command.args.remove(0);
}

fn display_string_list(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(", ")
    }
}



