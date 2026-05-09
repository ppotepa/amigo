import { ContextRow } from "../../../../ui/context-dock/ContextRow";
import { sceneContextIcon } from "../sceneContextIcons";
import type { SceneContextModel } from "../sceneContextTypes";

export function SceneInfoTab({ model }: { model: SceneContextModel }) {
  return (
    <div className="scene-detail-tab">
      <section className="scene-detail-section">
        <ContextRow title="Status" subtitle={model.sceneInfo.status} badge={model.sceneInfo.status} />
        <ContextRow title="Launcher" subtitle={model.sceneInfo.launcherVisible ? "visible" : "hidden"} />
        <ContextRow title="Entities" subtitle={`${model.sceneInfo.entityCount}`} />
        <ContextRow title="Scripts" subtitle={`${model.sceneInfo.scriptCount}`} />
        <ContextRow title="Assets" subtitle={`${model.sceneInfo.assetCount}`} />
      </section>

      <section className="scene-detail-section">
        <header className="scene-detail-section-header">
          <span>Asset Usage</span>
          <small>{model.sceneInfo.assetCount} assets</small>
        </header>
        {model.sceneInfo.assetGroups.length ? model.sceneInfo.assetGroups.map((group) => (
          <ContextRow
            key={group.id}
            icon={sceneContextIcon("asset")}
            title={group.label}
            subtitle={`${group.managedAssets.length} managed, ${group.rawFiles.length} raw`}
            badge={group.count}
            tone="blue"
          />
        )) : (
          <p className="muted workspace-note">No asset usage indexed for this scene.</p>
        )}
      </section>
    </div>
  );
}
