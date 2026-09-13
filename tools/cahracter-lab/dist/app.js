"use strict";
(() => {
  // src/anatomy-rig.ts
  var anatomyRig = {
    id: "body",
    type: "group",
    anchor: { x: 0, y: 0, z: 0 },
    children: [
      {
        id: "head",
        type: "group",
        anchor: { x: 0, y: 0, z: 0 },
        children: [
          {
            id: "face",
            type: "mass",
            anchor: { x: 0, y: -16, z: 0 },
            geometry: { strategy: "head.faceMorph" },
            visibilityMode: "always",
            fusionGroup: "skin",
            renderPass: "midMass",
            render: {
              material: "skin",
              zIndex: 20,
              primaryLighting: true,
              debugSamplePoints: true,
              outline: {
                contourClassName: "partOutline",
                contourZIndex: 100,
                silhouetteZIndex: 90
              },
              outputs: [
                { kind: "path", role: "fill", className: "surfaceBase" },
                { kind: "path", role: "shade", className: "surfaceShade", opacityMeta: "shadeOpacity" },
                { kind: "path", role: "light", className: "surfaceLight", opacityMeta: "lightOpacity" },
                { kind: "path", role: "wire", className: "wirePath" }
              ]
            }
          },
          {
            id: "nose",
            type: "mass",
            anchor: { x: 0, y: 8, z: 42 },
            geometry: { strategy: "head.noseMorph" },
            visibilityMode: "not-back",
            fusionGroup: "skin",
            renderPass: "nearMass",
            render: {
              material: "skin",
              zIndex: 30,
              outline: {
                contourClassName: "partOutline strong",
                contourZIndex: 101,
                silhouetteZIndex: 91
              },
              outputs: [
                { kind: "path", role: "fill", className: "svgNose" },
                { kind: "path", role: "wire", className: "wirePath" }
              ]
            },
            children: [
              {
                id: "noseHighlight",
                type: "detail",
                anchor: { x: 0, y: 0, z: 50 },
                geometry: {
                  strategy: "head.noseHighlight",
                  input: { parent: true }
                },
                visibilityMode: "front-side",
                renderPass: "detail",
                render: {
                  zIndex: 60,
                  outputs: [{ kind: "path", role: "detail", layer: "rig", className: "noseHighlight" }]
                }
              },
              {
                id: "nostril",
                type: "detail",
                anchor: { x: 0, y: 18, z: 48 },
                geometry: {
                  strategy: "head.nostril",
                  input: { parent: true }
                },
                visibilityMode: "front-side",
                renderPass: "detail",
                render: {
                  zIndex: 61,
                  outputs: [
                    {
                      kind: "ellipse",
                      role: "detail",
                      layer: "rig",
                      className: "nostril",
                      metaAttrs: ["cx", "cy", "rx", "ry"]
                    }
                  ]
                }
              }
            ]
          },
          {
            id: "earRight",
            type: "mass",
            anchor: { x: 76, y: 0, z: -8 },
            geometry: { strategy: "head.earMorph", side: "right" },
            visibilityMode: "always",
            fusionGroup: "skin",
            renderPass: "midMass",
            render: {
              material: "skin",
              zIndex: 25,
              outline: {
                contourClassName: "partOutline",
                contourZIndex: 102,
                silhouetteZIndex: 92
              },
              outputs: [
                { kind: "path", role: "fill", className: "surfaceBase" },
                { kind: "path", role: "shade", className: "surfaceShade", opacityMeta: "shadeOpacity" },
                { kind: "path", role: "light", className: "surfaceLight", opacityMeta: "lightOpacity" },
                { kind: "path", role: "wire", className: "wirePath" }
              ]
            },
            children: [
              {
                id: "earRightInner",
                type: "detail",
                anchor: { x: 78, y: 0, z: -4 },
                geometry: {
                  strategy: "head.earInnerCurve",
                  side: "right",
                  input: { parent: true }
                },
                visibilityMode: "front-side",
                renderPass: "detail",
                render: {
                  zIndex: 62,
                  outputs: [{ kind: "path", role: "detail", layer: "rig", className: "detailContour" }]
                }
              }
            ]
          },
          {
            id: "earLeft",
            type: "mass",
            anchor: { x: -76, y: 0, z: -8 },
            geometry: { strategy: "head.earMorph", side: "left" },
            visibilityMode: "always",
            fusionGroup: "skin",
            renderPass: "midMass",
            render: {
              material: "skin",
              zIndex: 25,
              outline: {
                contourClassName: "partOutline",
                contourZIndex: 103,
                silhouetteZIndex: 93
              },
              outputs: [
                { kind: "path", role: "fill", className: "surfaceBase" },
                { kind: "path", role: "shade", className: "surfaceShade", opacityMeta: "shadeOpacity" },
                { kind: "path", role: "light", className: "surfaceLight", opacityMeta: "lightOpacity" },
                { kind: "path", role: "wire", className: "wirePath" }
              ]
            },
            children: [
              {
                id: "earLeftInner",
                type: "detail",
                anchor: { x: -78, y: 0, z: -4 },
                geometry: {
                  strategy: "head.earInnerCurve",
                  side: "left",
                  input: { parent: true }
                },
                visibilityMode: "front-side",
                renderPass: "detail",
                render: {
                  zIndex: 62,
                  outputs: [{ kind: "path", role: "detail", layer: "rig", className: "detailContour" }]
                }
              }
            ]
          },
          {
            id: "mouth",
            type: "detail",
            anchor: { x: 0, y: 52, z: 36 },
            geometry: {
              strategy: "head.mouthLine",
              input: { sourcePartId: "face" }
            },
            visibilityMode: "not-back",
            renderPass: "detail",
            render: {
              zIndex: 55,
              outputs: [{ kind: "path", role: "detail", layer: "rig", className: "mouthLine" }]
            }
          }
        ]
      }
    ]
  };

  // src/render-primitive-generator.ts
  function primitiveAttrsFromGeometry(geometry, spec) {
    const attrs = { ...spec.attrs ?? {} };
    for (const key of spec.metaAttrs ?? []) {
      const value = geometry.meta?.[key];
      if (value !== void 0) attrs[key] = value;
    }
    return attrs;
  }
  function fallbackOutlinePolicy() {
    return {
      drawBody: true,
      drawContour: false,
      drawInner: false,
      drawSilhouette: false
    };
  }
  function resolveBodyVisibility(part, role, decision) {
    if (role === "detail") return decision.drawBody || decision.drawInner;
    if (role === "fill" || role === "shade" || role === "light" || role === "wire") return decision.drawBody;
    return true;
  }
  function generateBodyPartPrimitives(parts, policy) {
    const out = [];
    for (const part of parts) {
      const geometry = part.geometry;
      const outputs = part.source.render?.outputs ?? [];
      if (!geometry || outputs.length === 0) continue;
      const decision = policy?.parts[part.id] ?? fallbackOutlinePolicy();
      for (const [index, spec] of outputs.entries()) {
        if (spec.kind === "path" && !geometry.path) continue;
        if (spec.kind === "ellipse" && !geometry.meta) continue;
        const opacityValue = spec.opacityMeta ? geometry.meta?.[spec.opacityMeta] ?? spec.opacity ?? geometry.opacity ?? 1 : spec.opacity ?? geometry.opacity ?? 1;
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
          opacity
        });
      }
    }
    return out;
  }
  function collectPathMap(parts) {
    const map = {};
    for (const part of parts) {
      if (part.geometry?.path) map[part.id] = part.geometry.path;
    }
    return map;
  }
  function generateOutlinePrimitives(parts, specs) {
    const paths = collectPathMap(parts);
    const out = [];
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
        opacity: 1
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
        opacity: 1
      });
    }
    return out;
  }
  function generatePolicyOutlinePrimitives(parts, policy, overrideVisibility) {
    const specs = parts.filter((part) => Boolean(part.source.render?.outline)).map((part) => {
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
        silhouetteZIndex: outline.silhouetteZIndex
      };
    });
    return generateOutlinePrimitives(parts, specs);
  }

  // src/anatomy-render-primitives.ts
  function generateAnatomyRenderPrimitives(parts, policy) {
    return [
      ...generateBodyPartPrimitives(parts, policy),
      ...generatePolicyOutlinePrimitives(parts, policy)
    ];
  }
  function describePrimitiveOrder(primitives) {
    const labels = primitives.filter((primitive) => primitive.visible).map((primitive) => primitive.sourcePartId).filter((label, index, items) => items.indexOf(label) === index);
    return `order: ${labels.join(" -> ")}`;
  }

  // src/types.ts
  var NS = "http://www.w3.org/2000/svg";
  var PIVOT_X = 540;
  var PIVOT_Y = 540;
  var OUTLINE_MODES = {
    FULL: "FULL",
    ADAPTIVE: "ADAPTIVE",
    SILHOUETTE_ONLY: "SILHOUETTE_ONLY",
    PAINTERLY: "PAINTERLY"
  };

  // src/dom.ts
  function requireElement(id) {
    const element = document.getElementById(id);
    if (!element) throw new Error(`Missing element #${id}`);
    return element;
  }
  function createEl(name, attrs = {}) {
    const node = document.createElementNS(NS, name);
    for (const [key, value] of Object.entries(attrs)) node.setAttribute(key, value);
    return node;
  }
  function getUiRefs() {
    return {
      stage: requireElement("stage"),
      rigLayer: requireElement("rigLayer"),
      outlineLayer: requireElement("outlineLayer"),
      debugLayer: requireElement("debugLayer"),
      sourceGuides: requireElement("sourceGuides"),
      yaw: requireElement("yaw"),
      outlineMode: requireElement("outlineMode"),
      samples: requireElement("samples"),
      smooth: requireElement("smooth"),
      depth: requireElement("depth"),
      auto: requireElement("auto"),
      sources: requireElement("sources"),
      wire: requireElement("wire"),
      dots: requireElement("dots"),
      exportSvg: requireElement("exportSvg"),
      yawOut: requireElement("yawOut"),
      outlineOut: requireElement("outlineOut"),
      samplesOut: requireElement("samplesOut"),
      smoothOut: requireElement("smoothOut"),
      depthOut: requireElement("depthOut"),
      hudYaw: requireElement("hudYaw"),
      hudBlend: requireElement("hudBlend"),
      hudMode: requireElement("hudMode"),
      hudFusion: requireElement("hudFusion"),
      hudOutline: requireElement("hudOutline"),
      hudOrder: requireElement("hudOrder"),
      buttons: Array.from(document.querySelectorAll("[data-yaw]")),
      shadowStop1: requireElement("shadowStop1"),
      shadowStop2: requireElement("shadowStop2"),
      shadowStop3: requireElement("shadowStop3"),
      lightStop1: requireElement("lightStop1"),
      lightStop2: requireElement("lightStop2"),
      lightStop3: requireElement("lightStop3")
    };
  }

  // src/math.ts
  function clamp(value, min, max) {
    return Math.max(min, Math.min(max, value));
  }
  function smoothstep(t) {
    t = clamp(t, 0, 1);
    return t * t * (3 - 2 * t);
  }
  function lerp(a, b, t) {
    return a + (b - a) * t;
  }
  function remapClamp(x, a, b) {
    return clamp((x - a) / (b - a), 0, 1);
  }
  function pointLerp(a, b, t) {
    return { x: lerp(a.x, b.x, t), y: lerp(a.y, b.y, t) };
  }
  function lerpPoints(a, b, t) {
    return a.map((point, index) => pointLerp(point, b[index], t));
  }
  function keyframeTangent(frames, frameIndex, pointIndex) {
    const previous = frames[Math.max(0, frameIndex - 1)];
    const next = frames[Math.min(frames.length - 1, frameIndex + 1)];
    const dt = next.t - previous.t || 1;
    return {
      x: (next.points[pointIndex].x - previous.points[pointIndex].x) / dt,
      y: (next.points[pointIndex].y - previous.points[pointIndex].y) / dt
    };
  }
  function interpolatePointKeyframes(frames, value, curve = 0.55) {
    if (!frames.length) return [];
    if (value <= frames[0].t) return frames[0].points;
    if (value >= frames[frames.length - 1].t) return frames[frames.length - 1].points;
    const segmentIndex = Math.max(0, frames.findIndex((frame, index) => index > 0 && value <= frame.t) - 1);
    const a = frames[segmentIndex];
    const b = frames[segmentIndex + 1];
    const dt = b.t - a.t || 1;
    const u = clamp((value - a.t) / dt, 0, 1);
    const u2 = u * u;
    const u3 = u2 * u;
    const h00 = 2 * u3 - 3 * u2 + 1;
    const h10 = u3 - 2 * u2 + u;
    const h01 = -2 * u3 + 3 * u2;
    const h11 = u3 - u2;
    return a.points.map((point, index) => {
      const next = b.points[index];
      const tangentA = keyframeTangent(frames, segmentIndex, index);
      const tangentB = keyframeTangent(frames, segmentIndex + 1, index);
      const hermite = {
        x: h00 * point.x + h10 * dt * tangentA.x + h01 * next.x + h11 * dt * tangentB.x,
        y: h00 * point.y + h10 * dt * tangentA.y + h01 * next.y + h11 * dt * tangentB.y
      };
      return pointLerp(pointLerp(point, next, u), hermite, curve);
    });
  }
  function distanceSq(a, b) {
    const dx = a.x - b.x;
    const dy = a.y - b.y;
    return dx * dx + dy * dy;
  }
  function centroid(points) {
    const sum = points.reduce((acc, point) => ({ x: acc.x + point.x, y: acc.y + point.y }), { x: 0, y: 0 });
    return { x: sum.x / points.length, y: sum.y / points.length };
  }
  function signedArea(points) {
    let sum = 0;
    for (let i = 0; i < points.length; i++) {
      const a = points[i];
      const b = points[(i + 1) % points.length];
      sum += a.x * b.y - b.x * a.y;
    }
    return sum * 0.5;
  }
  function boundsOf(points) {
    const xs = points.map((point) => point.x);
    const ys = points.map((point) => point.y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    return { minX, minY, maxX, maxY, width: maxX - minX, height: maxY - minY };
  }
  function normalized(points) {
    const bounds = boundsOf(points);
    const width = bounds.width || 1;
    const height = bounds.height || 1;
    return points.map((point) => ({
      x: (point.x - bounds.minX) / width,
      y: (point.y - bounds.minY) / height
    }));
  }
  function rotatePoints(points, offset) {
    const length = points.length;
    const out = new Array(length);
    for (let i = 0; i < length; i++) out[i] = points[(i + offset) % length];
    return out;
  }
  function reversePoints(points) {
    return [...points].reverse();
  }
  function mirrorPoints(points, pivotX = PIVOT_X) {
    return points.map((point) => ({ x: pivotX * 2 - point.x, y: point.y }));
  }
  function pointsToPath(points, smoothAmount) {
    if (!points.length || points.length < 2) return "";
    if (smoothAmount <= 1e-3) {
      let d2 = `M ${points[0].x.toFixed(2)} ${points[0].y.toFixed(2)}`;
      for (let i = 1; i < points.length; i++) {
        d2 += ` L ${points[i].x.toFixed(2)} ${points[i].y.toFixed(2)}`;
      }
      return `${d2} Z`;
    }
    const k = smoothAmount / 6;
    let d = `M ${points[0].x.toFixed(2)} ${points[0].y.toFixed(2)}`;
    for (let i = 0; i < points.length; i++) {
      const p0 = points[(i - 1 + points.length) % points.length];
      const p1 = points[i];
      const p2 = points[(i + 1) % points.length];
      const p3 = points[(i + 2) % points.length];
      const c1 = { x: p1.x + (p2.x - p0.x) * k, y: p1.y + (p2.y - p0.y) * k };
      const c2 = { x: p2.x - (p3.x - p1.x) * k, y: p2.y - (p3.y - p1.y) * k };
      d += ` C ${c1.x.toFixed(2)} ${c1.y.toFixed(2)}, ${c2.x.toFixed(2)} ${c2.y.toFixed(2)}, ${p2.x.toFixed(2)} ${p2.y.toFixed(2)}`;
    }
    return `${d} Z`;
  }
  function applyPseudoDepth(points, yawRad, t, depth) {
    if (depth <= 0) return points;
    const side = Math.sign(Math.sin(yawRad)) || 1;
    const squeeze = 1 - depth * 0.09 * t;
    const parallax = side * depth * 13 * t;
    const verticalLift = -depth * 3.5 * t * Math.cos(yawRad);
    return points.map((point) => ({
      x: PIVOT_X + (point.x - PIVOT_X) * squeeze + parallax,
      y: PIVOT_Y + (point.y - PIVOT_Y) * (1 + depth * 0.012 * t) + verticalLift
    }));
  }

  // src/render.ts
  function ensureDots(ui, dots, count) {
    while (dots.length < count) {
      const dot = createEl("circle", { class: "dot", r: "3.4" });
      ui.debugLayer.appendChild(dot);
      dots.push(dot);
    }
    return dots;
  }
  function setGradientDirection(ui, points, side, t) {
    const bounds = boundsOf(points);
    const shadowNearX = side > 0 ? bounds.minX + bounds.width * 0.34 : bounds.maxX - bounds.width * 0.34;
    const lightNearX = side > 0 ? bounds.maxX - bounds.width * 0.18 : bounds.minX + bounds.width * 0.18;
    const blend = 0.22 + t * 0.22;
    const midY = bounds.minY + bounds.height * 0.52;
    const shadowY2 = bounds.maxY - bounds.height * 0.12;
    const lightY2 = bounds.maxY - bounds.height * 0.18;
    ui.shadowStop1.parentElement?.setAttribute("x1", shadowNearX.toFixed(2));
    ui.shadowStop1.parentElement?.setAttribute("y1", midY.toFixed(2));
    ui.shadowStop1.parentElement?.setAttribute("x2", (shadowNearX + side * bounds.width * blend).toFixed(2));
    ui.shadowStop1.parentElement?.setAttribute("y2", shadowY2.toFixed(2));
    ui.lightStop1.parentElement?.setAttribute("x1", lightNearX.toFixed(2));
    ui.lightStop1.parentElement?.setAttribute("y1", (bounds.minY + bounds.height * 0.18).toFixed(2));
    ui.lightStop1.parentElement?.setAttribute("x2", (lightNearX - side * bounds.width * 0.25).toFixed(2));
    ui.lightStop1.parentElement?.setAttribute("y2", lightY2.toFixed(2));
  }
  function injectInlineStyles(target) {
    const styleText = Array.from(document.styleSheets).map((sheet) => {
      try {
        return Array.from(sheet.cssRules).map((rule) => rule.cssText).join("\n");
      } catch {
        return "";
      }
    }).filter(Boolean).join("\n");
    if (!styleText) return;
    const style = document.createElementNS(NS, "style");
    style.textContent = styleText;
    target.insertBefore(style, target.firstChild);
  }

  // src/export.ts
  function exportCurrentSvg(ui, rerender, yaw) {
    rerender();
    const clone = ui.stage.cloneNode(true);
    clone.setAttribute("xmlns", NS);
    clone.setAttribute("width", "1080");
    clone.setAttribute("height", "1080");
    clone.querySelectorAll(".axisLine").forEach((node) => node.remove());
    injectInlineStyles(clone);
    const content = `<?xml version="1.0" encoding="UTF-8"?>
${new XMLSerializer().serializeToString(clone)}
`;
    const blob = new Blob([content], { type: "image/svg+xml" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `character-yaw-${Math.round(yaw)}.svg`;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 0);
  }

  // src/geometry/generated.ts
  var GENERATED_MASS_GEOMETRY_STRATEGIES = ["generated.softMass"];
  function numberParam(part, key, fallback) {
    const value = part.geometry?.params?.[key];
    return typeof value === "number" ? value : fallback;
  }
  function buildSoftMassContour(part, viewState) {
    const halfTop = numberParam(part, "topWidth", 48) * 0.5;
    const halfBottom = numberParam(part, "bottomWidth", 72) * 0.5;
    const height = numberParam(part, "height", 140);
    const shoulderDrop = numberParam(part, "shoulderDrop", height * 0.18);
    const waistLift = numberParam(part, "waistLift", height * 0.1);
    const profilePinch = numberParam(part, "profilePinch", 0.28);
    const sideShift = numberParam(part, "sideShift", 10) * viewState.side;
    const pinch = 1 - viewState.t * profilePinch;
    const top = halfTop * pinch;
    const bottom = halfBottom * pinch;
    const cx = part.anchor.x + sideShift;
    const topY = part.anchor.y - height * 0.5;
    const bottomY = part.anchor.y + height * 0.5;
    return [
      { x: cx - top, y: topY + shoulderDrop * 0.15 },
      { x: cx - top * 1.08, y: topY + shoulderDrop * 0.62 },
      { x: cx - bottom * 0.94, y: bottomY - waistLift * 0.55 },
      { x: cx - bottom, y: bottomY - waistLift * 0.08 },
      { x: cx - bottom * 0.34, y: bottomY },
      { x: cx + bottom * 0.34, y: bottomY },
      { x: cx + bottom, y: bottomY - waistLift * 0.08 },
      { x: cx + bottom * 0.94, y: bottomY - waistLift * 0.55 },
      { x: cx + top * 1.08, y: topY + shoulderDrop * 0.62 },
      { x: cx + top, y: topY + shoulderDrop * 0.15 },
      { x: cx + top * 0.28, y: topY },
      { x: cx - top * 0.28, y: topY }
    ];
  }
  function evaluateGeneratedMassGeometry(part, state, viewState, _yaw, yawRad) {
    if (!part.geometry) return null;
    switch (part.geometry.strategy) {
      case "generated.softMass": {
        const points = applyPseudoDepth(buildSoftMassContour(part, viewState), yawRad, viewState.t, state.depth);
        return {
          points,
          path: pointsToPath(points, state.smooth),
          bounds: boundsOf(points),
          opacity: 1,
          visible: true,
          meta: {
            shadeOpacity: (0.64 + viewState.t * 0.14 + viewState.back * 0.04).toFixed(3),
            lightOpacity: (0.56 + (1 - viewState.back) * 0.1).toFixed(3)
          }
        };
      }
      default:
        return null;
    }
  }
  var generatedMassGeometryResolverEntries = Object.fromEntries(
    GENERATED_MASS_GEOMETRY_STRATEGIES.map((strategy) => [strategy, evaluateGeneratedMassGeometry])
  );

  // src/rig.ts
  var SOURCE = {
    centerFace: "#CENTER #FACE",
    centerRightEar: "#CENTER #PRAWE",
    centerLeftEar: "#CENTER #LEWE",
    centerNose: "#CENTER #NOSE",
    angleFace: "#ANGLE #ANGLE_FACE",
    angleNose: "#ANGLE #ANGLE_NOSE",
    angleEar: "#ANGLE #ANGLE_EAR",
    profileFace: "#LEFT_PROFILE #FACE_PROFILE",
    profileNose: "#LEFT_PROFILE #NOSE-PROFILE",
    profileEar: "#LEFT_PROFILE #EAR_LEFT",
    backFace: "#BACK #BACK_FACE",
    backLeftEar: "#BACK #BACK_LEFT_EAR",
    backRightEar: "#BACK #BACK_RIGHT_EAR"
  };
  var EAR_DEFS = {
    earRight: { localSide: 1, frontSelector: SOURCE.centerRightEar, backSelector: SOURCE.backLeftEar, label: "right" },
    earLeft: { localSide: -1, frontSelector: SOURCE.centerLeftEar, backSelector: SOURCE.backRightEar, label: "left" }
  };
  function samplePath(path, count, mirrored = false) {
    const total = path.getTotalLength();
    const matrix = path.getCTM();
    let points = [];
    for (let i = 0; i < count; i++) {
      const point = path.getPointAtLength(i / count * total);
      if (matrix) {
        const worldPoint = new DOMPoint(point.x, point.y).matrixTransform(matrix);
        points.push({ x: worldPoint.x, y: worldPoint.y });
      } else {
        points.push({ x: point.x, y: point.y });
      }
    }
    if (mirrored) points = mirrorPoints(points);
    return { id: path.id || "path", length: total, points, center: centroid(points), area: signedArea(points) };
  }
  function sampleSelector(selector, count, mirrored = false) {
    const element = document.querySelector(selector);
    if (!element) throw new Error(`Brak path: ${selector}`);
    return samplePath(element, count, mirrored);
  }
  function alignTargetToSource(sourcePoints, targetPoints) {
    const length = sourcePoints.length;
    const sourceNorm = normalized(sourcePoints);
    const candidates = [targetPoints, reversePoints(targetPoints)];
    let best = null;
    for (const candidate of candidates) {
      const targetNorm = normalized(candidate);
      for (let offset = 0; offset < length; offset++) {
        let score = 0;
        for (let i = 0; i < length; i += 2) score += distanceSq(sourceNorm[i], targetNorm[(i + offset) % length]);
        if (!best || score < best.score) best = { score, reversed: candidate !== targetPoints, offset };
      }
    }
    const oriented = best?.reversed ? reversePoints(targetPoints) : targetPoints;
    return rotatePoints(oriented, best?.offset ?? 0);
  }
  function extremeIndex(points, axis) {
    let bestIndex = 0;
    for (let i = 1; i < points.length; i++) {
      const point = points[i];
      const best = points[bestIndex];
      const better = axis === "top" ? point.y < best.y : axis === "bottom" ? point.y > best.y : axis === "left" ? point.x < best.x : point.x > best.x;
      if (better) bestIndex = i;
    }
    return bestIndex;
  }
  function alignByAnchor(sourcePoints, targetPoints, anchor) {
    const base = alignTargetToSource(sourcePoints, targetPoints);
    const sourceIndex = extremeIndex(sourcePoints, anchor);
    const targetIndex = extremeIndex(base, anchor);
    return rotatePoints(base, ((sourceIndex - targetIndex) % base.length + base.length) % base.length);
  }
  function fitContourToReference(reference, candidate, fitX, fitY, anchor = "center") {
    const referenceBounds = boundsOf(reference);
    const candidateBounds = boundsOf(candidate);
    const referenceCenter = centroid(reference);
    const candidateCenter = centroid(candidate);
    const targetScaleX = candidateBounds.width > 0 ? referenceBounds.width / candidateBounds.width : 1;
    const targetScaleY = candidateBounds.height > 0 ? referenceBounds.height / candidateBounds.height : 1;
    const scaleX = 1 + (targetScaleX - 1) * fitX;
    const scaleY = 1 + (targetScaleY - 1) * fitY;
    const scaled = candidate.map((point) => ({
      x: candidateCenter.x + (point.x - candidateCenter.x) * scaleX,
      y: candidateCenter.y + (point.y - candidateCenter.y) * scaleY
    }));
    const scaledBounds = boundsOf(scaled);
    const scaledCenter = centroid(scaled);
    const dx = referenceCenter.x - scaledCenter.x;
    const dy = anchor === "bottom" ? referenceBounds.maxY - scaledBounds.maxY : referenceCenter.y - scaledCenter.y;
    return scaled.map((point) => ({ x: point.x + dx, y: point.y + dy }));
  }
  function deriveProfileNoseTarget(centerNose, quarterNose, rawProfileNose) {
    const alignedQuarter = alignByAnchor(centerNose, quarterNose, "top");
    const alignedProfile = alignByAnchor(centerNose, rawProfileNose, "top");
    const fittedProfile = fitContourToReference(alignedQuarter, alignedProfile, 0.45, 0.72);
    return lerpPoints(alignedQuarter, fittedProfile, 0.72);
  }
  function stabilizeHeadTarget(reference, candidate, widthFit) {
    return fitContourToReference(reference, candidate, widthFit, 0.92, "bottom");
  }
  function stabilizeEarTarget(reference, candidate) {
    return fitContourToReference(reference, candidate, 0.78, 0.86, "center");
  }
  function buildRig(side, samples) {
    const mirrored = side < 0;
    const centerFace = sampleSelector(SOURCE.centerFace, samples, false);
    const angleFace = sampleSelector(SOURCE.angleFace, samples, mirrored);
    const profileFace = sampleSelector(SOURCE.profileFace, samples, mirrored);
    const backFace = sampleSelector(SOURCE.backFace, samples, false);
    const centerNose = sampleSelector(SOURCE.centerNose, samples, false);
    const angleNose = sampleSelector(SOURCE.angleNose, samples, mirrored);
    const targetNose = sampleSelector(SOURCE.profileNose, samples, mirrored);
    return [
      {
        key: "face",
        source: centerFace.points,
        quarter: stabilizeHeadTarget(centerFace.points, alignByAnchor(centerFace.points, angleFace.points, "bottom"), 0.45),
        target: stabilizeHeadTarget(centerFace.points, alignByAnchor(centerFace.points, profileFace.points, "bottom"), 0.3),
        back: stabilizeHeadTarget(centerFace.points, alignByAnchor(centerFace.points, backFace.points, "bottom"), 0.68)
      },
      {
        key: "nose",
        source: centerNose.points,
        quarter: alignByAnchor(centerNose.points, angleNose.points, "top"),
        target: deriveProfileNoseTarget(centerNose.points, angleNose.points, targetNose.points)
      }
    ];
  }
  function buildEarRig(samples) {
    const angleLeft = sampleSelector(SOURCE.angleEar, samples, false);
    const angleRight = sampleSelector(SOURCE.angleEar, samples, true);
    const profileLeft = sampleSelector(SOURCE.profileEar, samples, false);
    const profileRight = sampleSelector(SOURCE.profileEar, samples, true);
    const rig = {};
    for (const [key, def] of Object.entries(EAR_DEFS)) {
      const front = sampleSelector(def.frontSelector, samples, false);
      const back = sampleSelector(def.backSelector, samples, false);
      rig[key] = {
        key,
        localSide: def.localSide,
        front: front.points,
        quarterLeft: stabilizeEarTarget(front.points, alignByAnchor(front.points, angleLeft.points, "top")),
        profileLeft: stabilizeEarTarget(front.points, alignByAnchor(front.points, profileLeft.points, "top")),
        back: stabilizeEarTarget(front.points, alignByAnchor(front.points, back.points, "top")),
        quarterRight: stabilizeEarTarget(front.points, alignByAnchor(front.points, angleRight.points, "top")),
        profileRight: stabilizeEarTarget(front.points, alignByAnchor(front.points, profileRight.points, "top"))
      };
    }
    return rig;
  }
  function computeEarState(key, localSide, yawRad) {
    const s = Math.sin(yawRad);
    const c = Math.cos(yawRad);
    const profile = Math.abs(s);
    const back = Math.max(0, -c);
    const depth = localSide * s;
    const screenX = PIVOT_X + localSide * 118 * c;
    const isNear = depth > 0.035;
    const isFar = depth < -0.035;
    const frontLike = Math.abs(depth) <= 0.035;
    const fusion = isNear ? 0.18 + 0.34 * smoothstep(remapClamp(profile, 0.62, 1)) : smoothstep(remapClamp(profile, 0.18, 0.72));
    return {
      key,
      localSide,
      depth,
      screenX,
      isNear,
      isFar,
      frontLike,
      profileT: smoothstep(profile),
      backT: smoothstep(back),
      fusion: clamp(fusion, 0, 1)
    };
  }
  function computeEarPoints(earRig, yawDeg) {
    const deg = (yawDeg % 360 + 360) % 360;
    return interpolatePointKeyframes(
      [
        { t: 0, points: earRig.front },
        { t: 45, points: earRig.quarterLeft },
        { t: 90, points: earRig.profileLeft },
        { t: 180, points: earRig.back },
        { t: 270, points: earRig.profileRight },
        { t: 315, points: earRig.quarterRight },
        { t: 360, points: earRig.front }
      ],
      deg,
      0.42
    );
  }
  function computeRigViewState(yawRad, yawDeg) {
    const s = Math.sin(yawRad);
    const c = Math.cos(yawRad);
    const side = s >= 0 ? 1 : -1;
    const profile = Math.abs(s);
    const back = Math.max(0, -c);
    let zone;
    if (back >= 0.82) zone = "BACK_PROXY";
    else if (back >= 0.45) zone = "REAR_TRANSITION";
    else if (profile >= 0.82) zone = "PROFILE";
    else if (profile >= 0.28) zone = "THREE_QUARTER";
    else zone = "FRONT";
    const t = smoothstep(profile);
    const noseFusion = zone === "BACK_PROXY" ? 1 : smoothstep(remapClamp(profile, 0.28, 0.68));
    const earRight = computeEarState("earRight", 1, yawRad);
    const earLeft = computeEarState("earLeft", -1, yawRad);
    const ears = [earRight, earLeft];
    const depthSorted = [...ears].sort((a, b) => a.depth - b.depth || a.localSide - b.localSide);
    const farEar = depthSorted[0];
    const nearEar = depthSorted[1];
    const showNose = zone !== "BACK_PROXY" && !(zone === "REAR_TRANSITION" && back >= 0.68);
    const showMouth = zone === "FRONT" || zone === "THREE_QUARTER";
    let noseMode;
    if (!showNose) noseMode = "HIDDEN";
    else if (noseFusion < 0.22) noseMode = "SEPARATE";
    else if (noseFusion < 0.82) noseMode = "MERGING";
    else noseMode = "FUSED";
    return {
      yawDeg,
      side,
      profile,
      back,
      zone,
      t,
      noseFusion,
      noseMode,
      ears,
      earRight,
      earLeft,
      nearEar,
      farEar,
      showNose,
      showMouth
    };
  }

  // src/geometry/head.ts
  var HEAD_MASS_GEOMETRY_STRATEGIES = ["head.faceMorph", "head.noseMorph", "head.earMorph"];
  var HEAD_DETAIL_GEOMETRY_STRATEGIES = [
    "head.mouthLine",
    "head.noseHighlight",
    "head.nostril",
    "head.earInnerCurve"
  ];
  function noseFacingScore(viewState) {
    const profilePresence = smoothstep(remapClamp(viewState.profile, 0.18, 0.82));
    const backFade = 1 - smoothstep(remapClamp(viewState.back, 0.12, 0.72));
    return profilePresence * backFade;
  }
  function earInnerFacingScore(viewState, isNear) {
    const profilePresence = smoothstep(remapClamp(viewState.profile, 0.1, 0.72));
    const nearBias = isNear ? 1 : 0.35;
    const backFade = 1 - smoothstep(remapClamp(viewState.back, 0.42, 0.72));
    return profilePresence * nearBias * backFade;
  }
  function getFaceRig(state, side) {
    const key = `${side}:${state.samples}`;
    if (!state.rigs.has(key)) state.rigs.set(key, buildRig(side, state.samples));
    return state.rigs.get(key);
  }
  function getEarRig(state) {
    const key = `${state.samples}`;
    if (!state.earRigs.has(key)) state.earRigs.set(key, buildEarRig(state.samples));
    return state.earRigs.get(key);
  }
  function localYawForSide(yaw, side) {
    return side > 0 ? yaw : 360 - yaw;
  }
  function interpolateRigPoints(pair, localYaw) {
    const frames = [
      { t: 0, points: pair.source },
      { t: 45, points: pair.quarter ?? pair.target },
      { t: 90, points: pair.target },
      ...pair.back ? [{ t: 180, points: pair.back }] : []
    ];
    return interpolatePointKeyframes(frames, localYaw, pair.back ? 0.5 : 0.35);
  }
  function evaluateHeadMassGeometry(part, state, viewState, yaw, yawRad) {
    if (!part.geometry) return null;
    const t = viewState.t;
    const localYaw = localYawForSide(yaw, viewState.side);
    switch (part.geometry.strategy) {
      case "head.faceMorph": {
        const pair = getFaceRig(state, viewState.side).find((item) => item.key === "face");
        if (!pair?.back) return null;
        const points = applyPseudoDepth(interpolateRigPoints(pair, localYaw), yawRad, t, state.depth * 0.55);
        return {
          points,
          path: pointsToPath(points, state.smooth),
          bounds: boundsOf(points),
          opacity: 1,
          visible: true,
          meta: {
            shadeOpacity: (0.75 + t * 0.12 + viewState.back * 0.08).toFixed(3),
            lightOpacity: (0.68 + t * 0.08 - viewState.back * 0.14).toFixed(3)
          }
        };
      }
      case "head.noseMorph": {
        const pair = getFaceRig(state, viewState.side).find((item) => item.key === "nose");
        if (!pair) return null;
        const points = applyPseudoDepth(interpolateRigPoints(pair, Math.min(localYaw, 90)), yawRad, t, state.depth * 0.45);
        return {
          points,
          path: pointsToPath(points, state.smooth),
          bounds: boundsOf(points),
          opacity: 1,
          visible: viewState.showNose
        };
      }
      case "head.earMorph": {
        const earKey = part.geometry.side === "left" ? "earLeft" : "earRight";
        const earState = viewState.ears.find((item) => item.key === earKey);
        if (!earState) return null;
        const points = applyPseudoDepth(computeEarPoints(getEarRig(state)[earKey], yaw), yawRad, t, state.depth * 0.28);
        return {
          points,
          path: pointsToPath(points, state.smooth),
          bounds: boundsOf(points),
          opacity: 1,
          visible: true,
          meta: {
            shadeOpacity: String((0.22 + 0.22 * (1 - earState.fusion) + 0.1 * (earState.isNear ? 1 : 0)) * (earState.isFar ? 0.72 : 1)),
            lightOpacity: String(0.28 + 0.12 * (1 - earState.fusion))
          }
        };
      }
      default:
        return null;
    }
  }
  function evaluateHeadDetailGeometry(part, parentGeometry, viewState, outlineMode) {
    if (!part.geometry) return null;
    switch (part.geometry.strategy) {
      case "head.mouthLine": {
        const facePoints = parentGeometry?.points;
        if (!facePoints) return null;
        const bounds = boundsOf(facePoints);
        const center = centroid(facePoints);
        const nearSign = viewState.side > 0 ? -1 : 1;
        const t = viewState.t;
        const mouthY = center.y + bounds.height * 0.23;
        const mouthW = lerp(bounds.width * 0.14, bounds.width * 0.09, t);
        const mouthTilt = viewState.side * t * bounds.height * 0.015;
        const mouthX = center.x + nearSign * t * bounds.width * 0.015;
        return {
          path: `M ${(mouthX - mouthW).toFixed(2)} ${(mouthY - mouthTilt).toFixed(2)} Q ${mouthX.toFixed(2)} ${(mouthY + bounds.height * 0.018).toFixed(2)}, ${(mouthX + mouthW).toFixed(2)} ${(mouthY + mouthTilt).toFixed(2)}`,
          bounds,
          visible: viewState.showMouth && viewState.profile < 0.96
        };
      }
      case "head.noseHighlight": {
        const nosePoints = parentGeometry?.points;
        if (!nosePoints) return null;
        const bounds = boundsOf(nosePoints);
        const center = centroid(nosePoints);
        const sign = viewState.side > 0 ? -1 : 1;
        const t = viewState.t;
        const facing = noseFacingScore(viewState);
        return {
          path: `M ${(center.x - sign * bounds.width * 0.1).toFixed(2)} ${(bounds.minY + bounds.height * 0.2).toFixed(2)} Q ${(center.x - sign * bounds.width * 0.18).toFixed(2)} ${center.y.toFixed(2)}, ${(center.x - sign * bounds.width * 0.1).toFixed(2)} ${(bounds.minY + bounds.height * 0.84).toFixed(2)}`,
          bounds,
          opacity: 0.05 + facing * 0.24 + t * 0.06,
          visible: viewState.showNose && outlineMode !== "SILHOUETTE_ONLY" && viewState.zone !== "BACK_PROXY" && facing > 0.16
        };
      }
      case "head.nostril": {
        const nosePoints = parentGeometry?.points;
        if (!nosePoints) return null;
        const bounds = boundsOf(nosePoints);
        const center = centroid(nosePoints);
        const sign = viewState.side > 0 ? -1 : 1;
        const t = viewState.t;
        const facing = noseFacingScore(viewState);
        return {
          bounds,
          opacity: 0.03 + facing * 0.18 + t * 0.05,
          visible: viewState.showNose && outlineMode !== "SILHOUETTE_ONLY" && viewState.zone !== "BACK_PROXY" && facing > 0.2,
          meta: {
            cx: (center.x + sign * bounds.width * 0.17).toFixed(2),
            cy: (bounds.minY + bounds.height * 0.72).toFixed(2),
            rx: Math.max(1, bounds.width * lerp(0.032, 0.055, t)).toFixed(2),
            ry: Math.max(0.6, bounds.height * lerp(0.015, 0.025, t)).toFixed(2)
          }
        };
      }
      case "head.earInnerCurve": {
        const earPoints = parentGeometry?.points;
        if (!earPoints) return null;
        const bounds = boundsOf(earPoints);
        const cx = bounds.minX + bounds.width * 0.52;
        const y1 = bounds.minY + bounds.height * 0.22;
        const y2 = bounds.minY + bounds.height * 0.78;
        const localSide = part.geometry.side === "right" ? 1 : -1;
        const earState = viewState.ears.find((item) => item.key === (part.geometry?.side === "right" ? "earRight" : "earLeft"));
        if (!earState) return null;
        const curve = bounds.width * (localSide > 0 ? -0.18 : 0.18) * (earState.depth >= 0 ? 1 : -1);
        const facing = earInnerFacingScore(viewState, earState.isNear);
        return {
          path: `M ${cx.toFixed(2)} ${y1.toFixed(2)} C ${(cx + curve).toFixed(2)} ${(y1 + bounds.height * 0.18).toFixed(2)}, ${(cx - curve * 0.65).toFixed(2)} ${(y2 - bounds.height * 0.18).toFixed(2)}, ${cx.toFixed(2)} ${y2.toFixed(2)}`,
          bounds,
          visible: outlineMode !== "SILHOUETTE_ONLY" && facing > 0.22
        };
      }
      default:
        return null;
    }
  }
  var headMassGeometryResolverEntries = Object.fromEntries(
    HEAD_MASS_GEOMETRY_STRATEGIES.map((strategy) => [strategy, evaluateHeadMassGeometry])
  );
  var headDetailGeometryResolverEntries = Object.fromEntries(
    HEAD_DETAIL_GEOMETRY_STRATEGIES.map((strategy) => [strategy, evaluateHeadDetailGeometry])
  );

  // src/geometry/packs.ts
  var geometryResolverPacks = [
    {
      mass: headMassGeometryResolverEntries,
      detail: headDetailGeometryResolverEntries
    },
    {
      mass: generatedMassGeometryResolverEntries
    }
  ];

  // src/geometry/registry.ts
  function combineResolverEntries(...packs) {
    return Object.assign({}, ...packs);
  }
  var massGeometryResolvers = combineResolverEntries(
    ...geometryResolverPacks.map((pack) => pack.mass ?? {})
  );
  var detailGeometryResolvers = combineResolverEntries(
    ...geometryResolverPacks.map((pack) => pack.detail ?? {})
  );

  // src/geometry.ts
  function evaluateMassGeometry(part, state, viewState, yaw, yawRad) {
    const strategy = part.geometry?.strategy;
    if (!strategy) return null;
    const resolver = massGeometryResolvers[strategy];
    return resolver ? resolver(part, state, viewState, yaw, yawRad) : null;
  }
  function evaluateDetailGeometry(part, parentGeometry, viewState, outlineMode) {
    const strategy = part.geometry?.strategy;
    if (!strategy) return null;
    const resolver = detailGeometryResolvers[strategy];
    return resolver ? resolver(part, parentGeometry, viewState, outlineMode) : null;
  }

  // src/head-debug.ts
  function matrixToAttr(path) {
    const matrix = path.getCTM();
    if (!matrix) return null;
    return `matrix(${matrix.a} ${matrix.b} ${matrix.c} ${matrix.d} ${matrix.e} ${matrix.f})`;
  }
  function renderHeadSources(ui, side, backT = 0) {
    ui.sourceGuides.innerHTML = "";
    const centerSelectors = ["#CENTER #FACE", "#CENTER #NOSE", side > 0 ? "#CENTER #PRAWE" : "#CENTER #LEWE"];
    const profileSelectors = ["#LEFT_PROFILE #FACE_PROFILE", "#LEFT_PROFILE #NOSE-PROFILE", "#LEFT_PROFILE #EAR_LEFT"];
    for (const selector of centerSelectors) {
      const path = document.querySelector(selector);
      if (!path) continue;
      const ghost = createEl("path", { class: "ghostPath centerGhost", d: path.getAttribute("d") ?? "" });
      const transform = matrixToAttr(path);
      if (transform) ghost.setAttribute("transform", transform);
      ui.sourceGuides.appendChild(ghost);
    }
    for (const selector of profileSelectors) {
      const path = document.querySelector(selector);
      if (!path) continue;
      const ghost = createEl("path", { class: "ghostPath profileGhost", d: path.getAttribute("d") ?? "" });
      const transform = matrixToAttr(path);
      if (transform && side < 0) {
        ghost.setAttribute("transform", `translate(1080,0) scale(-1,1) ${transform}`);
      } else if (transform) {
        ghost.setAttribute("transform", transform);
      } else if (side < 0) {
        ghost.setAttribute("transform", "translate(1080,0) scale(-1,1)");
      }
      ui.sourceGuides.appendChild(ghost);
    }
    ui.sourceGuides.style.opacity = String(1 - smoothstep(remapClamp(backT, 0.45, 1)) * 0.58);
  }
  function formatHeadMode(viewState) {
    const sideName = viewState.side > 0 ? "LEFT" : "RIGHT";
    if (viewState.zone === "FRONT") return `zone: FRONT / nose ${viewState.noseMode}`;
    if (viewState.zone === "BACK_PROXY") return "zone: BACK_PROXY / nose+mouth state-hidden";
    return `zone: ${viewState.zone} ${sideName} / nose ${viewState.noseMode}`;
  }
  function formatHeadFusion(viewState, skinFusionCount) {
    return `fusion: nose ${viewState.noseFusion.toFixed(2)} / R ${viewState.earRight.fusion.toFixed(2)} / L ${viewState.earLeft.fusion.toFixed(2)} / skin ${skinFusionCount}`;
  }

  // src/render-primitives.ts
  var PASS_ORDER = {
    farMass: 0,
    midMass: 1,
    nearMass: 2,
    detail: 3,
    outline: 4
  };
  function sortRenderPrimitives(items) {
    return [...items].sort((a, b) => {
      const passDiff = PASS_ORDER[a.pass] - PASS_ORDER[b.pass];
      if (passDiff !== 0) return passDiff;
      const zDiff = a.zIndex - b.zIndex;
      if (zDiff !== 0) return zDiff;
      return a.depth - b.depth;
    });
  }

  // src/primitive-renderer.ts
  function layerFor(ui, layer) {
    if (layer === "outline") return ui.outlineLayer;
    if (layer === "debug") return ui.debugLayer;
    return ui.rigLayer;
  }
  function createPrimitiveElement(primitive) {
    return createEl(primitive.kind);
  }
  function ensureSilhouetteGroup(ui) {
    let group = ui.outlineLayer.querySelector('[data-primitive-group="silhouette-union"]');
    if (!group) {
      group = createEl("g");
      group.setAttribute("data-primitive-group", "silhouette-union");
      group.setAttribute("class", "masterSilhouetteUnion");
      ui.outlineLayer.appendChild(group);
    }
    return group;
  }
  function applyPrimitive(element, primitive) {
    element.setAttribute("data-primitive-id", primitive.id);
    if (primitive.role === "silhouette") {
      element.removeAttribute("class");
    } else if (primitive.className) {
      element.setAttribute("class", primitive.className);
    } else {
      element.removeAttribute("class");
    }
    element.style.display = primitive.visible ? "inline" : "none";
    element.style.opacity = String(primitive.opacity ?? 1);
    if (primitive.kind === "path" && primitive.path) {
      element.setAttribute("d", primitive.path);
    }
    if (primitive.kind === "circle" && primitive.point) {
      element.setAttribute("cx", primitive.point.x.toFixed(2));
      element.setAttribute("cy", primitive.point.y.toFixed(2));
    }
    for (const [key, value] of Object.entries(primitive.attrs ?? {})) {
      element.setAttribute(key, String(value));
    }
  }
  var PrimitiveSvgRenderer = class {
    constructor(ui) {
      this.ui = ui;
      this.elements = /* @__PURE__ */ new Map();
      this.silhouetteGroup = ensureSilhouetteGroup(ui);
    }
    render(primitives) {
      const sorted = sortRenderPrimitives(primitives);
      const live = new Set(sorted.map((item) => item.id));
      for (const [id, element] of this.elements) {
        if (!live.has(id)) {
          element.remove();
          this.elements.delete(id);
        }
      }
      for (const primitive of sorted) {
        let element = this.elements.get(primitive.id);
        if (!element || element.tagName.toLowerCase() !== primitive.kind) {
          element?.remove();
          element = createPrimitiveElement(primitive);
          this.elements.set(primitive.id, element);
        }
        applyPrimitive(element, primitive);
        if (primitive.role === "silhouette") {
          this.silhouetteGroup.appendChild(element);
        } else {
          layerFor(this.ui, primitive.layer).appendChild(element);
        }
      }
    }
  };

  // src/policies/fusion.ts
  function computeFusionPolicy(part, ctx) {
    if (!part.fusionGroup) return { group: null, t: 0 };
    return {
      group: part.fusionGroup,
      t: smoothstep(remapClamp(ctx.side, 0.28, 0.68))
    };
  }

  // src/policies/layer.ts
  function computeRenderPassPolicy(part, depth) {
    if (part.renderPass && part.type !== "mass") return part.renderPass;
    if (part.type === "detail") return "detail";
    if (part.type === "contour") return "outline";
    if (part.type === "mass") {
      if (depth < -18) return "farMass";
      if (depth > 18) return "nearMass";
      if (part.renderPass === "farMass" || part.renderPass === "nearMass") return part.renderPass;
      return "midMass";
    }
    if (part.renderPass) return part.renderPass;
    if (part.type === "group") return "midMass";
    if (depth < -12) return "farMass";
    if (depth > 12) return "nearMass";
    return "midMass";
  }
  function computeRenderPasses(parts) {
    const order = {
      farMass: 0,
      midMass: 1,
      nearMass: 2,
      detail: 3,
      outline: 4
    };
    return [...parts].filter((part) => part.visible && part.type !== "group").sort((a, b) => {
      const passDiff = order[a.renderPass] - order[b.renderPass];
      if (passDiff !== 0) return passDiff;
      const zDiff = (a.source.render?.zIndex ?? 0) - (b.source.render?.zIndex ?? 0);
      if (zDiff !== 0) return zDiff;
      return a.depth - b.depth;
    });
  }

  // src/policies/visibility.ts
  function computeVisibilityPolicy(part, ctx) {
    switch (part.visibilityMode ?? "always") {
      case "front-only":
        return ctx.back < 0.06 && ctx.side < 0.22;
      case "front-profile":
        return ctx.back < 0.38;
      case "front-side":
        return ctx.back < 0.5;
      case "not-back":
        return ctx.back < 0.62;
      default:
        return true;
    }
  }

  // src/scene-graph.ts
  function computeViewContext(angle) {
    const theta = angle * Math.PI / 180;
    const c = Math.cos(theta);
    const s = Math.sin(theta);
    return {
      angle,
      theta,
      c,
      s,
      front: Math.max(0, c),
      back: Math.max(0, -c),
      side: Math.abs(s),
      dir: s >= 0 ? 1 : -1,
      profileT: smoothstep(Math.abs(s)),
      backT: smoothstep(Math.max(0, -c))
    };
  }
  function projectPoint2_5D(anchor, ctx) {
    return {
      x: PIVOT_X + (anchor.x * ctx.c + anchor.z * ctx.s),
      y: PIVOT_Y + anchor.y,
      depth: anchor.z * ctx.c - anchor.x * ctx.s
    };
  }
  function evaluateBodyPart(part, ctx) {
    const projected = projectPoint2_5D(part.anchor, ctx);
    const children = (part.children ?? []).map((child) => evaluateBodyPart(child, ctx));
    return {
      id: part.id,
      type: part.type,
      source: part,
      projected,
      visible: computeVisibilityPolicy(part, ctx),
      depth: projected.depth,
      renderPass: computeRenderPassPolicy(part, projected.depth),
      fusion: computeFusionPolicy(part, ctx),
      geometry: null,
      children
    };
  }
  function flattenParts(root) {
    const out = [root];
    for (const child of root.children) out.push(...flattenParts(child));
    return out;
  }
  function computeFusionGroups(parts) {
    const groups = /* @__PURE__ */ new Map();
    for (const part of parts) {
      if (!part.fusion.group) continue;
      const list = groups.get(part.fusion.group) ?? [];
      list.push(part);
      groups.set(part.fusion.group, list);
    }
    return groups;
  }
  function computeRenderPasses2(parts) {
    return computeRenderPasses(parts);
  }

  // src/head-outline-policy.ts
  function computeEarPolicy(earState, viewState, mode) {
    const isProfileFar = earState.isFar && viewState.profile > 0.64;
    const isBackSide = viewState.back > 0.62;
    if (mode === OUTLINE_MODES.FULL) {
      return { drawBody: true, drawContour: true, drawInner: true, drawSilhouette: true };
    }
    if (mode === OUTLINE_MODES.SILHOUETTE_ONLY) {
      return { drawBody: true, drawContour: false, drawInner: false, drawSilhouette: !isProfileFar };
    }
    if (mode === OUTLINE_MODES.PAINTERLY) {
      return {
        drawBody: true,
        drawContour: earState.isNear && viewState.profile < 0.38 && !isBackSide,
        drawInner: false,
        drawSilhouette: !isProfileFar
      };
    }
    return {
      drawBody: true,
      drawContour: earState.isNear ? viewState.profile < 0.86 || isBackSide : viewState.profile < 0.36 || isBackSide,
      drawInner: earState.isNear && viewState.profile < 0.68 && viewState.back < 0.58,
      drawSilhouette: !isProfileFar
    };
  }
  function partPolicy(overrides = {}) {
    return {
      drawBody: true,
      drawContour: false,
      drawInner: false,
      drawSilhouette: false,
      ...overrides
    };
  }
  function noseFacingScore2(viewState) {
    const profilePresence = smoothstep(remapClamp(viewState.profile, 0.18, 0.82));
    const backFade = 1 - smoothstep(remapClamp(viewState.back, 0.12, 0.72));
    return profilePresence * backFade;
  }
  function computeOutlinePolicy(viewState, mode) {
    const showFace = true;
    const showNose = viewState.showNose;
    const showMouth = viewState.showMouth;
    const showEarInner = viewState.zone !== "REAR_TRANSITION" && viewState.zone !== "BACK_PROXY" && viewState.profile < 0.72;
    const showNoseDetail = viewState.zone === "FRONT" || viewState.zone === "THREE_QUARTER" || viewState.zone === "PROFILE" && viewState.back < 0.08;
    const showNoseDetailStrong = showNoseDetail && noseFacingScore2(viewState) > 0.5;
    const adaptiveNoseDetail = showNoseDetail && noseFacingScore2(viewState) > 0.12;
    const earPolicies = {
      earRight: computeEarPolicy(viewState.earRight, viewState, mode),
      earLeft: computeEarPolicy(viewState.earLeft, viewState, mode)
    };
    if (mode === OUTLINE_MODES.FULL) {
      return {
        drawMasterSilhouette: true,
        parts: {
          face: partPolicy({ drawBody: showFace, drawContour: true, drawSilhouette: true }),
          nose: partPolicy({ drawBody: showNose, drawContour: showNose, drawSilhouette: showNose }),
          earRight: earPolicies.earRight,
          earLeft: earPolicies.earLeft,
          mouth: partPolicy({ drawBody: showMouth, drawContour: false }),
          noseHighlight: partPolicy({ drawBody: showNose }),
          nostril: partPolicy({ drawBody: showNose }),
          earRightInner: partPolicy({ drawBody: earPolicies.earRight.drawInner && showEarInner }),
          earLeftInner: partPolicy({ drawBody: earPolicies.earLeft.drawInner && showEarInner })
        }
      };
    }
    if (mode === OUTLINE_MODES.SILHOUETTE_ONLY) {
      return {
        drawMasterSilhouette: true,
        parts: {
          face: partPolicy({ drawBody: showFace, drawSilhouette: true }),
          nose: partPolicy({ drawBody: showNose, drawSilhouette: showNose }),
          earRight: partPolicy({ drawBody: earPolicies.earRight.drawBody, drawSilhouette: earPolicies.earRight.drawSilhouette }),
          earLeft: partPolicy({ drawBody: earPolicies.earLeft.drawBody, drawSilhouette: earPolicies.earLeft.drawSilhouette }),
          mouth: partPolicy({ drawBody: false }),
          noseHighlight: partPolicy({ drawBody: false }),
          nostril: partPolicy({ drawBody: false }),
          earRightInner: partPolicy({ drawBody: false }),
          earLeftInner: partPolicy({ drawBody: false })
        }
      };
    }
    if (mode === OUTLINE_MODES.PAINTERLY) {
      return {
        drawMasterSilhouette: true,
        parts: {
          face: partPolicy({ drawBody: showFace, drawSilhouette: true }),
          nose: partPolicy({
            drawBody: showNose,
            drawContour: showNose && viewState.noseFusion < 0.18 && viewState.zone !== "PROFILE",
            drawSilhouette: showNose
          }),
          earRight: earPolicies.earRight,
          earLeft: earPolicies.earLeft,
          mouth: partPolicy({ drawBody: showMouth && viewState.profile < 0.82 }),
          noseHighlight: partPolicy({ drawBody: showNoseDetailStrong }),
          nostril: partPolicy({ drawBody: showNoseDetailStrong }),
          earRightInner: partPolicy({ drawBody: false }),
          earLeftInner: partPolicy({ drawBody: false })
        }
      };
    }
    return {
      drawMasterSilhouette: true,
      parts: {
        face: partPolicy({ drawBody: showFace, drawSilhouette: true }),
        nose: partPolicy({
          drawBody: showNose,
          drawContour: showNose && viewState.noseFusion < 0.55 && viewState.zone !== "REAR_TRANSITION",
          drawSilhouette: showNose
        }),
        earRight: earPolicies.earRight,
        earLeft: earPolicies.earLeft,
        mouth: partPolicy({ drawBody: showMouth }),
        noseHighlight: partPolicy({ drawBody: adaptiveNoseDetail }),
        nostril: partPolicy({ drawBody: adaptiveNoseDetail }),
        earRightInner: partPolicy({ drawBody: earPolicies.earRight.drawInner && showEarInner }),
        earLeftInner: partPolicy({ drawBody: earPolicies.earLeft.drawInner && showEarInner })
      }
    };
  }

  // src/app.ts
  function collectParentIds(root, parentId = null, out = /* @__PURE__ */ new Map()) {
    out.set(root.id, parentId);
    for (const child of root.children) collectParentIds(child, root.id, out);
    return out;
  }
  function resolveGeometryInput(part, geometryById, parentById) {
    const input = part.source.geometry?.input;
    if (input?.sourcePartId) return geometryById.get(input.sourcePartId) ?? null;
    if (input?.parent) {
      const parentId = parentById.get(part.id);
      return parentId ? geometryById.get(parentId) ?? null : null;
    }
    return null;
  }
  var CharacterLabApp = class {
    constructor() {
      this.ui = getUiRefs();
      this.primitiveRenderer = new PrimitiveSvgRenderer(this.ui);
      this.bodyTree = anatomyRig;
      this.state = {
        yaw: 0,
        samples: Number(this.ui.samples.value),
        smooth: Number(this.ui.smooth.value),
        depth: Number(this.ui.depth.value),
        outlineMode: this.ui.outlineMode.value,
        auto: false,
        rigs: /* @__PURE__ */ new Map(),
        earRigs: /* @__PURE__ */ new Map(),
        dots: [],
        lastTime: 0,
        dragging: false,
        dragStartX: 0,
        dragStartYaw: 0
      };
      this.tick = (time) => {
        if (!this.state.lastTime) this.state.lastTime = time;
        const dt = Math.min(64, time - this.state.lastTime);
        this.state.lastTime = time;
        if (this.state.auto) this.setYaw(this.state.yaw + dt * 0.035);
        requestAnimationFrame(this.tick);
      };
    }
    init() {
      this.bindEvents();
      this.rebuild();
      requestAnimationFrame(this.tick);
    }
    bindEvents() {
      this.ui.yaw.addEventListener("input", (event) => this.setYaw(event.target.value));
      this.ui.outlineMode.addEventListener("input", () => {
        this.state.outlineMode = this.ui.outlineMode.value;
        this.render();
      });
      this.ui.samples.addEventListener("input", () => this.rebuild());
      this.ui.smooth.addEventListener("input", () => this.rebuild());
      this.ui.depth.addEventListener("input", () => this.rebuild());
      this.ui.auto.addEventListener("input", () => {
        this.state.auto = this.ui.auto.checked;
      });
      this.ui.sources.addEventListener("input", () => this.render());
      this.ui.wire.addEventListener("input", () => this.render());
      this.ui.dots.addEventListener("input", () => this.render());
      this.ui.exportSvg.addEventListener("click", () => exportCurrentSvg(this.ui, () => this.render(), this.state.yaw));
      this.ui.buttons.forEach((button) => button.addEventListener("click", () => this.setYaw(Number(button.dataset.yaw))));
      this.ui.stage.addEventListener("pointerdown", (event) => {
        this.state.dragging = true;
        this.state.dragStartX = event.clientX;
        this.state.dragStartYaw = this.state.yaw;
        this.ui.stage.setPointerCapture(event.pointerId);
      });
      this.ui.stage.addEventListener("pointermove", (event) => {
        if (!this.state.dragging) return;
        const dx = event.clientX - this.state.dragStartX;
        this.setYaw(this.state.dragStartYaw + dx * 0.5);
      });
      this.ui.stage.addEventListener("pointerup", (event) => {
        this.state.dragging = false;
        this.ui.stage.releasePointerCapture(event.pointerId);
      });
      this.ui.stage.addEventListener("pointercancel", () => {
        this.state.dragging = false;
      });
    }
    setYaw(value) {
      this.state.yaw = (Number(value) % 360 + 360) % 360;
      this.render();
    }
    rebuild() {
      this.state.samples = Number(this.ui.samples.value);
      this.state.smooth = Number(this.ui.smooth.value);
      this.state.depth = Number(this.ui.depth.value);
      this.state.rigs.clear();
      this.state.earRigs.clear();
      this.render();
    }
    attachGeometry(flatParts, parentById, viewState, yaw, yawRad) {
      const geometryById = /* @__PURE__ */ new Map();
      for (const part of flatParts) {
        if (part.type !== "mass") continue;
        const geometry = evaluateMassGeometry(part.source, this.state, viewState, yaw, yawRad);
        part.geometry = geometry;
        if (geometry) geometryById.set(part.id, geometry);
      }
      for (const part of flatParts) {
        if (part.type !== "detail") continue;
        const inputGeometry = resolveGeometryInput(part, geometryById, parentById);
        const geometry = evaluateDetailGeometry(part.source, inputGeometry, viewState, this.state.outlineMode);
        part.geometry = geometry;
        if (geometry) geometryById.set(part.id, geometry);
      }
      return geometryById;
    }
    render() {
      const yaw = (this.state.yaw % 360 + 360) % 360;
      const rad = yaw * Math.PI / 180;
      const viewContext = computeViewContext(yaw);
      const viewState = computeRigViewState(rad, yaw);
      const policy = computeOutlinePolicy(viewState, this.state.outlineMode);
      const evaluatedTree = evaluateBodyPart(this.bodyTree, viewContext);
      const flatParts = flattenParts(evaluatedTree);
      const parentById = collectParentIds(evaluatedTree);
      this.attachGeometry(flatParts, parentById, viewState, yaw, rad);
      const fusionGroups = computeFusionGroups(flatParts);
      const renderList = computeRenderPasses2(flatParts);
      const primitives = generateAnatomyRenderPrimitives(renderList, policy);
      this.primitiveRenderer.render(primitives);
      const lightingSource = renderList.find((part) => part.source.render?.primaryLighting && part.geometry?.points)?.geometry?.points ?? null;
      const sampleSource = renderList.find((part) => part.source.render?.debugSamplePoints && part.geometry?.points)?.geometry?.points ?? [];
      if (lightingSource) setGradientDirection(this.ui, lightingSource, viewState.side, viewState.t);
      this.state.dots = ensureDots(this.ui, this.state.dots, sampleSource.length);
      sampleSource.forEach((point, index) => {
        this.state.dots[index].setAttribute("cx", point.x.toFixed(2));
        this.state.dots[index].setAttribute("cy", point.y.toFixed(2));
      });
      const centerPct = Math.round((1 - viewState.t) * 100);
      const profilePct = Math.round(viewState.t * 100);
      const backPct = Math.round(viewState.back * 100);
      const skinFusion = fusionGroups.get("skin")?.length ?? 0;
      this.ui.yaw.value = String(Math.round(yaw));
      this.ui.yawOut.textContent = `${Math.round(yaw)}\xB0`;
      this.ui.outlineOut.textContent = this.state.outlineMode;
      this.ui.samplesOut.textContent = `${this.state.samples}`;
      this.ui.smoothOut.textContent = this.state.smooth.toFixed(2);
      this.ui.depthOut.textContent = this.state.depth.toFixed(2);
      this.ui.hudYaw.textContent = `yaw: ${Math.round(yaw)}\xB0`;
      this.ui.hudBlend.textContent = `blend: CENTER ${centerPct}% -> PROFILE ${profilePct}% / BACK ${backPct}%`;
      this.ui.hudMode.textContent = formatHeadMode(viewState);
      this.ui.hudFusion.textContent = formatHeadFusion(viewState, skinFusion);
      this.ui.hudOutline.textContent = `outline: ${this.state.outlineMode}`;
      this.ui.hudOrder.textContent = describePrimitiveOrder(primitives);
      this.ui.stage.classList.toggle("showSources", this.ui.sources.checked);
      this.ui.stage.classList.toggle("showWire", this.ui.wire.checked);
      this.ui.stage.classList.toggle("showDots", this.ui.dots.checked);
      this.ui.buttons.forEach((button) => {
        const value = Number(button.dataset.yaw);
        button.classList.toggle("active", Math.abs(value - Math.round(yaw)) < 1);
      });
      renderHeadSources(this.ui, viewState.side, viewContext.back);
    }
  };

  // src/main.ts
  var app = new CharacterLabApp();
  app.init();
})();
