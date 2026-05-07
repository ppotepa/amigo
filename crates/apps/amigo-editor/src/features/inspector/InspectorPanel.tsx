import { useEffect, useState } from "react";
import type {
  AssetRegistryDto,
  EditorModDetailsDto,
  EditorProjectFileDto,
  ManagedAssetDto,
} from "../../api/dto";
import { createSpritesheetRuleset, getAssetRegistry } from "../../api/editorApi";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import { assetToTarget, projectFileToTarget } from "../../editor-targets";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { SelectionProperties } from "../../properties/SelectionProperties";
import type { EditorSelection } from "../../properties/propertiesTypes";
import type { YamlSourceRef } from "../files/yamlSourceRefs";

export function InspectorPanel({
  context,
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return (
    <Inspector
      details={services.details ?? null}
      sessionId={context.sessionId ?? undefined}
      onRefreshProjectTree={services.onProjectTreeRefresh}
      onSelectAsset={(asset) => services.activateEditorTarget?.(assetToTarget(asset), "select")}
      onSelectFile={(file) => services.activateEditorTarget?.(projectFileToTarget(file), "select")}
      onShowYaml={services.targetBridge?.showYamlView}
      onApplyEditorCommand={services.applyEditorCommand}
      selection={services.selection ?? { kind: "empty" }}
      selectedAsset={services.selectedAsset ?? null}
    />
  );
}

function Inspector({
  details,
  sessionId,
  onRefreshProjectTree,
  onSelectAsset,
  onSelectFile,
  onShowYaml,
  onApplyEditorCommand,
  selection,
  selectedAsset,
}: {
  details: EditorModDetailsDto | null;
  sessionId?: string;
  onRefreshProjectTree?: () => void;
  onSelectAsset?: (asset: ManagedAssetDto) => void;
  onSelectFile?: (file: EditorProjectFileDto) => void;
  onShowYaml?: (source: YamlSourceRef) => void;
  onApplyEditorCommand?: WorkspaceRuntimeServices["applyEditorCommand"];
  selection: EditorSelection;
  selectedAsset: ManagedAssetDto | null;
}) {
  const [registry, setRegistry] = useState<AssetRegistryDto | null>(null);
  const [registryError, setRegistryError] = useState<string | null>(null);
  const [rulesetBusy, setRulesetBusy] = useState(false);
  const [rulesetError, setRulesetError] = useState<string | null>(null);

  useEffect(() => {
    if (!sessionId || !selectedAsset) {
      setRegistry(null);
      setRegistryError(null);
      setRulesetError(null);
      return;
    }
    let alive = true;
    void getAssetRegistry(sessionId)
      .then((next) => {
        if (alive) setRegistry(next.modId === details?.id ? next : null);
      })
      .catch((error) => {
        if (alive) setRegistryError(error instanceof Error ? error.message : String(error));
      });
    return () => {
      alive = false;
    };
  }, [details?.id, sessionId, selectedAsset?.assetKey]);

  async function handleAddRuleset() {
    if (!sessionId || !selectedAsset) return;
    setRulesetBusy(true);
    setRulesetError(null);
    try {
      await createSpritesheetRuleset(sessionId, {
        spritesheetAssetKey: selectedAsset.assetKey,
      });
      const next = await getAssetRegistry(sessionId);
      setRegistry(next.modId === details?.id ? next : null);
      onRefreshProjectTree?.();
    } catch (error) {
      setRulesetError(error instanceof Error ? error.message : String(error));
    } finally {
      setRulesetBusy(false);
    }
  }

  return (
    <div className="dock-scroll">
      <SelectionProperties
        selection={selection}
        context={{
          assetRegistry: registry,
          assetRegistryError: registryError,
          details,
          rulesetBusy,
          rulesetError,
          sessionId,
          onAddSpritesheetRuleset: handleAddRuleset,
          onSelectAsset,
          onSelectFile,
          onShowYaml,
          onApplyEditorCommand,
        }}
      />
    </div>
  );
}
