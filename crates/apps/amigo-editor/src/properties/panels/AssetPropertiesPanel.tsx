import { Plus } from "lucide-react";
import type { AssetRegistryDto, ManagedAssetDto, RawAssetFileDto } from "../../api/dto";
import { projectFileFromManagedAsset, projectFileFromRawAsset } from "../../assets/assetProjectFiles";
import { assetVisualForKind } from "../../assets/assetVisualRegistry";
import type { AssetSelection, PropertiesContext } from "../propertiesTypes";

type RelationGroup = {
  key: string;
  label: string;
  order: number;
  items: RelationItem[];
};

type RelationItem = {
  key: string;
  label: string;
  detail: string;
  kind: string;
  status: string;
  asset?: ManagedAssetDto;
  rawFile?: RawAssetFileDto;
};

export function AssetPropertiesPanel({
  context,
  selection,
}: {
  context: PropertiesContext;
  selection: AssetSelection;
}) {
  const asset = selection.asset;
  const showRulesets = asset.domain === "spritesheet" && asset.role === "family";
  const rulesets = showRulesets && context.assetRegistry
    ? context.assetRegistry.managedAssets.filter((candidate) => candidate.parentKey === asset.assetKey && candidate.kind.includes("ruleset"))
    : [];

  return (
    <>
      <section className="workspace-section">
        <h3>Asset</h3>
        <dl className="kv-list">
          <dt>Label</dt>
          <dd>{asset.label}</dd>
          <dt>Kind</dt>
          <dd>{asset.kind}</dd>
          <dt>Domain</dt>
          <dd>{asset.domain}</dd>
          <dt>Role</dt>
          <dd>{asset.role}</dd>
          <dt>Status</dt>
          <dd>{asset.status}</dd>
          <dt>Descriptor</dt>
          <dd title={asset.descriptorPath}>{asset.descriptorRelativePath}</dd>
          <dt>Parent</dt>
          <dd title={asset.parentKey ?? undefined}>{asset.parentKey ?? "none"}</dd>
        </dl>
      </section>

      {showRulesets ? (
        <AssetRulesetsSection
          asset={asset}
          busy={context.rulesetBusy}
          error={context.rulesetError}
          rulesets={rulesets}
          sessionReady={Boolean(context.sessionId)}
          onAdd={context.onAddSpritesheetRuleset}
          onSelectAsset={context.onSelectAsset}
          onSelectFile={context.onSelectFile}
        />
      ) : null}

      {context.assetRegistryError ? <p className="muted workspace-note">{context.assetRegistryError}</p> : null}
      <AssetRelationSection
        empty="none"
        registry={context.assetRegistry}
        title="References"
        values={asset.references}
        onSelectAsset={context.onSelectAsset}
        onSelectFile={context.onSelectFile}
      />
      <AssetRelationSection
        empty="none"
        registry={context.assetRegistry}
        title="Used By"
        values={asset.usedBy}
        onSelectAsset={context.onSelectAsset}
        onSelectFile={context.onSelectFile}
      />

      {asset.diagnostics.length ? (
        <section className="workspace-section">
          <h3>Diagnostics</h3>
          <div className="workspace-diagnostic-list">
            {asset.diagnostics.map((diagnostic, index) => (
              <div key={`${diagnostic.code}:${index}`} className="workspace-row">
                <span className={`badge diagnostic-${diagnostic.level}`}>{diagnostic.level}</span>
                <span>
                  <strong>{diagnostic.code}</strong>
                  <small>{diagnostic.message}</small>
                </span>
              </div>
            ))}
          </div>
        </section>
      ) : null}
    </>
  );
}

function AssetRulesetsSection({
  asset,
  busy,
  error,
  rulesets,
  sessionReady,
  onAdd,
  onSelectAsset,
  onSelectFile,
}: {
  asset: ManagedAssetDto;
  busy: boolean;
  error: string | null;
  rulesets: ManagedAssetDto[];
  sessionReady: boolean;
  onAdd?: (asset: ManagedAssetDto) => Promise<void>;
  onSelectAsset?: (asset: ManagedAssetDto) => void;
  onSelectFile?: (file: ReturnType<typeof projectFileFromManagedAsset>) => void;
}) {
  return (
    <section className="workspace-section asset-relation-panel">
      <header>
        <h3>Rulesets</h3>
        <button className="workspace-mini-button" type="button" disabled={!sessionReady || busy || !onAdd} onClick={() => void onAdd?.(asset)}>
          <Plus size={12} />
          Add
        </button>
      </header>
      {error ? <p className="muted workspace-note">{error}</p> : null}
      {rulesets.length ? (
        <div className="asset-relation-list">
          {rulesets.map((ruleset) => (
            <button
              key={ruleset.assetKey}
              type="button"
              className="workspace-row"
              onClick={() => {
                onSelectAsset?.(ruleset);
                onSelectFile?.(projectFileFromManagedAsset(ruleset));
              }}
            >
              <span className={`dock-icon asset-status-icon ${assetVisualForKind(ruleset.kind).tone} ${statusClass(ruleset.status)}`}>
                {assetVisualForKind(ruleset.kind).icon}
              </span>
              <span>
                <strong>{ruleset.label}</strong>
                <small>{ruleset.descriptorRelativePath}</small>
              </span>
              <small className="asset-row-status">{ruleset.status}</small>
            </button>
          ))}
        </div>
      ) : (
        <p className="muted workspace-note">{busy ? "Creating ruleset..." : "No rulesets yet."}</p>
      )}
    </section>
  );
}

function AssetRelationSection({
  empty,
  registry,
  title,
  values,
  onSelectAsset,
  onSelectFile,
}: {
  empty: string;
  registry: AssetRegistryDto | null;
  title: string;
  values: string[];
  onSelectAsset?: (asset: ManagedAssetDto) => void;
  onSelectFile?: (file: ReturnType<typeof projectFileFromManagedAsset> | ReturnType<typeof projectFileFromRawAsset>) => void;
}) {
  const groups = buildRelationGroups(values, registry);
  return (
    <section className="workspace-section asset-relation-panel">
      <header>
        <h3>{title}</h3>
      </header>
      {groups.length ? (
        <div className="asset-relation-list">
          {groups.map((group) => (
            <div key={group.key} className="asset-relation-group">
              <h4>{group.label}</h4>
              {group.items.map((item) => (
                <button
                  key={item.key}
                  type="button"
                  className="workspace-row"
                  disabled={!item.asset && !item.rawFile}
                  onClick={() => {
                    if (item.asset) {
                      onSelectAsset?.(item.asset);
                      onSelectFile?.(projectFileFromManagedAsset(item.asset));
                    } else if (item.rawFile) {
                      onSelectFile?.(projectFileFromRawAsset(item.rawFile));
                    }
                  }}
                >
                  <span className={`dock-icon asset-status-icon ${assetVisualForKind(item.kind).tone} ${statusClass(item.status)}`}>
                    {assetVisualForKind(item.kind).icon}
                  </span>
                  <span>
                    <strong>{item.label}</strong>
                    <small>{item.detail}</small>
                  </span>
                  <small className="asset-row-status">{item.status}</small>
                </button>
              ))}
            </div>
          ))}
        </div>
      ) : (
        <p className="muted workspace-note">{empty}</p>
      )}
    </section>
  );
}

function buildRelationGroups(values: string[], registry: AssetRegistryDto | null): RelationGroup[] {
  const groups = new Map<string, RelationGroup>();
  for (const value of values) {
    const item = relationItem(value, registry);
    const bucket = relationBucket(item);
    const group = groups.get(bucket.key) ?? { ...bucket, items: [] };
    group.items.push(item);
    groups.set(bucket.key, group);
  }

  return [...groups.values()]
    .sort((left, right) => left.order - right.order || left.label.localeCompare(right.label))
    .map((group) => ({
      ...group,
      items: [...group.items].sort((left, right) => left.label.localeCompare(right.label) || left.key.localeCompare(right.key)),
    }));
}

function relationItem(value: string, registry: AssetRegistryDto | null): RelationItem {
  const asset = registry?.managedAssets.find((candidate) => candidate.assetKey === value);
  if (asset) {
    return {
      key: value,
      label: asset.label || asset.assetId || asset.assetKey,
      detail: asset.descriptorRelativePath,
      kind: asset.kind,
      status: asset.status,
      asset,
    };
  }

  const rawFile = registry?.rawFiles.find((candidate) => candidate.relativePath === value);
  if (rawFile) {
    return {
      key: value,
      label: rawFile.relativePath.split("/").pop() ?? rawFile.relativePath,
      detail: rawFile.relativePath,
      kind: rawFile.mediaType,
      status: rawFile.orphan ? "orphan" : "valid",
      rawFile,
    };
  }

  return {
    key: value,
    label: value,
    detail: value,
    kind: "reference",
    status: registry ? "missing" : "unresolved",
  };
}

function relationBucket(item: RelationItem): Omit<RelationGroup, "items"> {
  if (item.rawFile) return { key: "raw", label: "Raw Sources", order: 70 };
  if (item.status === "unresolved") return { key: "unresolved", label: "Unresolved", order: 89 };
  if (!item.asset || item.status === "missing") return { key: "missing", label: "Missing", order: 90 };

  if (item.asset.kind.includes("tileset") && !item.asset.kind.includes("ruleset")) {
    return { key: "tilesets", label: "Tilesets", order: 20 };
  }
  if (item.asset.kind.includes("ruleset")) return { key: "rulesets", label: "Rulesets", order: 30 };
  if (item.asset.kind.includes("animation")) return { key: "animations", label: "Animations", order: 40 };

  switch (item.asset.domain) {
    case "scene":
      return { key: "scenes", label: "Scenes", order: 10 };
    case "spritesheet":
      return { key: "spritesheets", label: "Spritesheets", order: 15 };
    case "tilemap":
      return { key: "tilemaps", label: "Tilemaps", order: 50 };
    case "audio":
      return { key: "audio", label: "Audio", order: 60 };
    case "font":
      return { key: "fonts", label: "Fonts", order: 61 };
    case "script":
      return { key: "scripts", label: "Scripts", order: 62 };
    case "raw":
      return { key: "raw", label: "Raw Sources", order: 70 };
    default:
      return { key: "unknown", label: "Unknown", order: 95 };
  }
}

function statusClass(status: string): string {
  if (status === "error") return "asset-status-error";
  if (status === "missing" || status === "missingSource") return "asset-status-missingSource";
  if (status === "warning" || status === "orphan" || status === "unresolved") return "asset-status-warning";
  return "asset-status-valid";
}
