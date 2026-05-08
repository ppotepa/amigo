import { Fragment } from "react";
import type {
  EditorResolvedAssetRefDto,
  EditorResolvedPropertyValueDto,
  EditorSceneComponentInstanceDto,
} from "../../api/dto";
import type { EditorTargetComposedSection } from "../../editor-targets/editorTargetSectionComposer";
import type { ResolvedEditorTarget } from "../../editor-targets/editorTargetTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import {
  findMetadataTraitDescriptor,
  traitEditorControls,
  traitPatchOps,
  traitPropertyGroups,
  type EditorMetadataTraitPropertyGroupDescriptorDto,
} from "./editorMetadataTypes";

export type TraitSectionRendererProps = {
  section: EditorTargetComposedSection;
  target: ResolvedEditorTarget;
  services: WorkspaceRuntimeServices;
};

export function TraitSectionRenderer({
  section,
  target,
  services,
}: TraitSectionRendererProps) {
  const trait = findMetadataTraitDescriptor(services.metadataCatalog, section.traitKind);
  const properties = propertiesForTrait(target, section.traitKind);
  const assetRefs = assetRefsForTrait(target, section.traitKind);
  const propertyGroups = trait ? traitPropertyGroups(trait) : [];
  const controls = trait ? traitEditorControls(trait) : [];
  const patchOps = trait ? traitPatchOps(trait) : [];

  return (
    <section className="target-context-section trait-section">
      <header className="trait-section-header">
        <div>
          <p className="item-context-eyebrow">{section.traitKind}</p>
          <h3>{section.label}</h3>
        </div>
        <span className="badge badge-muted">{section.placement}</span>
      </header>

      {section.description ? (
        <p className="muted workspace-note">{section.description}</p>
      ) : null}

      {trait?.description && trait.description !== section.description ? (
        <p className="muted workspace-note">{trait.description}</p>
      ) : null}

      {propertyGroups.length ? (
        <TraitPropertyGroups groups={propertyGroups} properties={properties} />
      ) : properties.length ? (
        <TraitPropertyList properties={properties} />
      ) : null}

      {assetRefs.length ? <TraitAssetRefs assetRefs={assetRefs} /> : null}
      {controls.length ? <TraitControls controls={controls} /> : null}
      {patchOps.length ? <TraitPatchOps patchOps={patchOps} /> : null}
    </section>
  );
}

function componentsForTarget(target: ResolvedEditorTarget): EditorSceneComponentInstanceDto[] {
  const selection = target.selection;

  if (selection.kind === "component") {
    return [selection.component];
  }

  if (selection.kind === "entity") {
    return selection.entity.components ?? [];
  }

  return [];
}

function propertiesForTrait(
  target: ResolvedEditorTarget,
  traitKind: string,
): EditorResolvedPropertyValueDto[] {
  return [
    ...summaryPropertiesForTrait(target, traitKind),
    ...componentsForTarget(target).flatMap((component) =>
      component.properties.filter((property) => property.traitKind === traitKind),
    ),
  ];
}

function assetRefsForTrait(
  target: ResolvedEditorTarget,
  traitKind: string,
): EditorResolvedAssetRefDto[] {
  return componentsForTarget(target).flatMap((component) =>
    component.assetRefs.filter((assetRef) => assetRef.traitKind === traitKind),
  );
}

function summaryPropertiesForTrait(
  target: ResolvedEditorTarget,
  traitKind: string,
): EditorResolvedPropertyValueDto[] {
  const selection = target.selection;

  if (selection.kind === "scene") {
    const scene = selection.scene;
    if (traitKind === "SceneDocument") {
      return [
        readOnlySummaryProperty("id", "Scene ID", scene.id, traitKind, "scene.identity"),
        readOnlySummaryProperty("label", "Label", scene.label, traitKind, "scene.identity"),
        readOnlySummaryProperty("documentPath", "YAML", scene.documentPath, traitKind, "scene.document"),
      ];
    }

    if (traitKind === "HasScripts") {
      return [
        readOnlySummaryProperty("scriptPath", "Script", scene.scriptPath ?? "none", traitKind, "scene.scripts"),
      ];
    }
  }

  if (selection.kind === "entity" || selection.kind === "component") {
    const entity = selection.kind === "entity" ? selection.entity : selection.entity;

    if (traitKind === "HasIdentity") {
      return [
        readOnlySummaryProperty("id", "Entity ID", entity.id, traitKind, "entity.identity"),
        readOnlySummaryProperty("name", "Name", entity.name, traitKind, "entity.identity"),
        readOnlySummaryProperty("tags", "Tags", entity.tags, traitKind, "entity.identity"),
        readOnlySummaryProperty("groups", "Groups", entity.groups, traitKind, "entity.identity"),
      ];
    }

    if (traitKind === "HasVisibility") {
      return [
        readOnlySummaryProperty("visible", "Visible", entity.visible, traitKind, "entity.visibility"),
        readOnlySummaryProperty(
          "simulationEnabled",
          "Simulation",
          entity.simulationEnabled,
          traitKind,
          "entity.visibility",
        ),
        readOnlySummaryProperty(
          "collisionEnabled",
          "Collision",
          entity.collisionEnabled,
          traitKind,
          "entity.visibility",
        ),
      ];
    }

    if (traitKind === "HasComponents") {
      return [
        readOnlySummaryProperty("componentCount", "Components", entity.componentCount, traitKind, "entity.components"),
        readOnlySummaryProperty("componentTypes", "Component Types", entity.componentTypes, traitKind, "entity.components"),
      ];
    }
  }

  if (selection.kind === "component") {
    const component = selection.component;

    if (traitKind === "GenericEditable") {
      return [
        readOnlySummaryProperty("typeName", "Type", component.typeName, traitKind, "component.identity"),
        readOnlySummaryProperty("componentIndex", "Index", component.componentIndex, traitKind, "component.identity"),
        readOnlySummaryProperty("yamlPath", "YAML Path", component.yamlPath, traitKind, "component.identity"),
        readOnlySummaryProperty(
          "descriptorKind",
          "Descriptor",
          component.descriptorKind ?? "none",
          traitKind,
          "component.identity",
        ),
      ];
    }

    if (traitKind === "HasAssetRefs") {
      return [
        readOnlySummaryProperty("assetRefCount", "Asset Refs", component.assetRefs.length, traitKind, "assetRefs.summary"),
      ];
    }
  }

  return [];
}

function readOnlySummaryProperty(
  path: string,
  label: string,
  value: unknown,
  traitKind: string,
  group: string,
): EditorResolvedPropertyValueDto {
  return {
    path,
    label,
    value,
    traitKind,
    group,
    valueKind: "summary",
    editor: "ReadOnly",
    access: "ReadOnly",
    exists: true,
    editable: false,
  };
}

function TraitPropertyGroups({
  groups,
  properties,
}: {
  groups: EditorMetadataTraitPropertyGroupDescriptorDto[];
  properties: EditorResolvedPropertyValueDto[];
}) {
  const declaredGroupIds = new Set(groups.map((group) => group.id));
  const propertyGroups = [
    ...groups,
    ...Array.from(new Set(properties.map((property) => property.group ?? "metadata.properties")))
      .filter((groupId) => !declaredGroupIds.has(groupId))
      .map((groupId) => ({
        id: groupId,
        label: groupId,
        description: "",
      })),
  ];

  return (
    <div className="trait-property-groups">
      {propertyGroups.map((group) => {
        const groupProperties = properties.filter((property) => property.group === group.id);
        if (!groupProperties.length) return null;

        return (
          <section className="trait-property-group" key={group.id}>
            <h4>{group.label}</h4>
            {group.description ? (
              <p className="muted workspace-note">{group.description}</p>
            ) : null}
            <TraitPropertyList properties={groupProperties} />
          </section>
        );
      })}
    </div>
  );
}

function TraitPropertyList({
  properties,
}: {
  properties: EditorResolvedPropertyValueDto[];
}) {
  return (
    <dl className="kv-list trait-property-list">
      {properties.map((property) => (
        <Fragment key={property.path}>
          <dt title={property.path}>{property.label}</dt>
          <dd>{formatTraitValue(property.value)}</dd>
        </Fragment>
      ))}
    </dl>
  );
}

function TraitAssetRefs({ assetRefs }: { assetRefs: EditorResolvedAssetRefDto[] }) {
  return (
    <section className="trait-asset-refs">
      <h4>Asset References</h4>
      <dl className="kv-list">
        {assetRefs.map((assetRef) => (
          <Fragment key={assetRef.fieldPath}>
            <dt title={assetRef.fieldPath}>{assetRef.fieldPath}</dt>
            <dd>
              <span>{assetRef.value ?? "none"}</span>
              <span className="badge badge-muted">{assetRef.domain}</span>
              {assetRef.required ? <span className="badge badge-warning">required</span> : null}
            </dd>
          </Fragment>
        ))}
      </dl>
    </section>
  );
}

function TraitControls({ controls }: { controls: string[] }) {
  return (
    <section className="trait-controls">
      <h4>Editor Controls</h4>
      <div className="item-context-badges">
        {controls.map((control) => (
          <span className="badge badge-muted" key={control}>
            {control}
          </span>
        ))}
      </div>
    </section>
  );
}

function TraitPatchOps({ patchOps }: { patchOps: string[] }) {
  return (
    <section className="trait-patch-ops">
      <h4>Patch Operations</h4>
      <div className="item-context-badges">
        {patchOps.map((patchOp) => (
          <span className="badge badge-muted" key={patchOp}>
            {patchOp}
          </span>
        ))}
      </div>
    </section>
  );
}

function formatTraitValue(value: unknown): string {
  if (value === null || value === undefined) return "none";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);

  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}
