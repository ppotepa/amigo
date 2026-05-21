import { type EvaluatedBodyPart, type GeometryPayload, type OutlinePartPolicy, type OutlinePolicy } from "./types";
import { type PrimitiveOutputSpec, type RenderPrimitive } from "./render-primitives";

export type OutlinePrimitiveSpec = {
  sourcePartId: string;
  visible?: boolean;
  drawContour?: boolean;
  contourClassName?: string;
  contourZIndex?: number;
  drawSilhouette?: boolean;
  silhouetteClassName?: string;
  silhouetteZIndex?: number;
};

export type OutlineVisibilityOverride = (
  part: EvaluatedBodyPart,
  decision: OutlinePartPolicy,
  policy: OutlinePolicy,
) => { drawContour?: boolean; drawSilhouette?: boolean } | null;

function primitiveAttrsFromGeometry(geometry: GeometryPayload, spec: PrimitiveOutputSpec): Record<string, string | number | boolean> {
  const attrs: Record<string, string | number | boolean> = { ...(spec.attrs ?? {}) };
  for (const key of spec.metaAttrs ?? []) {
    const value = geometry.meta?.[key];
    if (value !== undefined) attrs[key] = value;
  }
  return attrs;
}

function fallbackOutlinePolicy(): OutlinePartPolicy {
  return {
    drawBody: true,
    drawContour: false,
    drawInner: false,
    drawSilhouette: false,
  };
}

function resolveBodyVisibility(part: EvaluatedBodyPart, role: RenderPrimitive["role"], decision: OutlinePartPolicy): boolean {
  if (role === "detail") return decision.drawBody || decision.drawInner;
  if (role === "fill" || role === "shade" || role === "light" || role === "wire") return decision.drawBody;
  return true;
}

export function generateBodyPartPrimitives(parts: EvaluatedBodyPart[], policy?: OutlinePolicy): RenderPrimitive[] {
  const out: RenderPrimitive[] = [];

  for (const part of parts) {
    const geometry = part.geometry;
    const outputs = part.source.render?.outputs ?? [];
    if (!geometry || outputs.length === 0) continue;
    const decision = policy?.parts[part.id] ?? fallbackOutlinePolicy();

    for (const [index, spec] of outputs.entries()) {
      if (spec.kind === "path" && !geometry.path) continue;
      if (spec.kind === "ellipse" && !geometry.meta) continue;

      const opacityValue = spec.opacityMeta
        ? geometry.meta?.[spec.opacityMeta] ?? spec.opacity ?? geometry.opacity ?? 1
        : spec.opacity ?? geometry.opacity ?? 1;
      const opacity = typeof opacityValue === "boolean" ? Number(opacityValue) : opacityValue;

      out.push({
        id: `${part.id}:${spec.id ?? spec.role}:${index}`,
        sourcePartId: part.id,
        kind: spec.kind,
        role: spec.role,
        layer: spec.layer ?? "rig",
        pass: spec.pass ?? part.renderPass,
        zIndex: (part.source.render?.zIndex ?? 0) + (spec.zIndexOffset ?? 0),
        depth: part.depth,
        visible: part.visible && geometry.visible !== false && resolveBodyVisibility(part, spec.role, decision),
        className: spec.className,
        path: geometry.path,
        attrs: primitiveAttrsFromGeometry(geometry, spec),
        opacity,
      });
    }
  }

  return out;
}

function collectPathMap(parts: EvaluatedBodyPart[]): Record<string, string> {
  const map: Record<string, string> = {};
  for (const part of parts) {
    if (part.geometry?.path) map[part.id] = part.geometry.path;
  }
  return map;
}

export function generateOutlinePrimitives(parts: EvaluatedBodyPart[], specs: OutlinePrimitiveSpec[]): RenderPrimitive[] {
  const paths = collectPathMap(parts);
  const out: RenderPrimitive[] = [];

  for (const spec of specs) {
    const path = paths[spec.sourcePartId];
    if (!path) continue;

    out.push({
      id: `outline:${spec.sourcePartId}`,
      sourcePartId: spec.sourcePartId,
      kind: "path",
      role: "contour",
      layer: "outline",
      pass: "outline",
      zIndex: spec.contourZIndex ?? 100,
      depth: 0,
      visible: Boolean(spec.visible) && Boolean(spec.drawContour),
      className: spec.contourClassName ?? "partOutline",
      path,
      opacity: 1,
    });

    out.push({
      id: `silhouette:${spec.sourcePartId}`,
      sourcePartId: spec.sourcePartId,
      kind: "path",
      role: "silhouette",
      layer: "outline",
      pass: "outline",
      zIndex: spec.silhouetteZIndex ?? 90,
      depth: 0,
      visible: Boolean(spec.visible) && Boolean(spec.drawSilhouette),
      className: spec.silhouetteClassName ?? "masterSilhouettePath",
      path,
      opacity: 1,
    });
  }

  return out;
}

export function generatePolicyOutlinePrimitives(
  parts: EvaluatedBodyPart[],
  policy: OutlinePolicy,
  overrideVisibility?: OutlineVisibilityOverride,
): RenderPrimitive[] {
  const specs: OutlinePrimitiveSpec[] = parts
    .filter(part => Boolean(part.source.render?.outline))
    .map(part => {
      const outline = part.source.render?.outline ?? {};
      const decision = policy.parts[part.id] ?? fallbackOutlinePolicy();
      const override = overrideVisibility?.(part, decision, policy);

      return {
        sourcePartId: part.id,
        visible: part.visible && part.geometry?.visible !== false,
        drawContour: override?.drawContour ?? decision.drawContour,
        contourClassName: outline.contourClassName,
        contourZIndex: outline.contourZIndex,
        drawSilhouette: override?.drawSilhouette ?? (policy.drawMasterSilhouette && decision.drawSilhouette),
        silhouetteClassName: outline.silhouetteClassName,
        silhouetteZIndex: outline.silhouetteZIndex,
      };
    });

  return generateOutlinePrimitives(parts, specs);
}
