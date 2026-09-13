import { anatomyRig } from "./anatomy-rig";
import { generateAnatomyRenderPrimitives, describePrimitiveOrder } from "./anatomy-render-primitives";
import { getUiRefs } from "./dom";
import { exportCurrentSvg } from "./export";
import { evaluateDetailGeometry, evaluateMassGeometry } from "./geometry";
import { renderHeadSources, formatHeadFusion, formatHeadMode } from "./head-debug";
import { PrimitiveSvgRenderer } from "./primitive-renderer";
import { ensureDots, setGradientDirection } from "./render";
import { computeRigViewState } from "./rig";
import { computeFusionGroups, computeRenderPasses, computeViewContext, evaluateBodyPart, flattenParts } from "./scene-graph";
import { computeOutlinePolicy } from "./head-outline-policy";
import { type AppState, type EvaluatedBodyPart, type GeometryPayload } from "./types";

function collectParentIds(
  root: EvaluatedBodyPart,
  parentId: string | null = null,
  out = new Map<string, string | null>(),
): Map<string, string | null> {
  out.set(root.id, parentId);
  for (const child of root.children) collectParentIds(child, root.id, out);
  return out;
}

function resolveGeometryInput(
  part: EvaluatedBodyPart,
  geometryById: Map<string, GeometryPayload>,
  parentById: Map<string, string | null>,
): GeometryPayload | null {
  const input = part.source.geometry?.input;
  if (input?.sourcePartId) return geometryById.get(input.sourcePartId) ?? null;
  if (input?.parent) {
    const parentId = parentById.get(part.id);
    return parentId ? geometryById.get(parentId) ?? null : null;
  }
  return null;
}

export class CharacterLabApp {
  private readonly ui = getUiRefs();
  private readonly primitiveRenderer = new PrimitiveSvgRenderer(this.ui);
  private readonly bodyTree = anatomyRig;
  private readonly state: AppState = {
    yaw: 0,
    samples: Number(this.ui.samples.value),
    smooth: Number(this.ui.smooth.value),
    depth: Number(this.ui.depth.value),
    outlineMode: this.ui.outlineMode.value as AppState["outlineMode"],
    auto: false,
    rigs: new Map(),
    earRigs: new Map(),
    dots: [],
    lastTime: 0,
    dragging: false,
    dragStartX: 0,
    dragStartYaw: 0,
  };

  init(): void {
    this.bindEvents();
    this.rebuild();
    requestAnimationFrame(this.tick);
  }

  private readonly tick = (time: number): void => {
    if (!this.state.lastTime) this.state.lastTime = time;
    const dt = Math.min(64, time - this.state.lastTime);
    this.state.lastTime = time;
    if (this.state.auto) this.setYaw(this.state.yaw + dt * 0.035);
    requestAnimationFrame(this.tick);
  };

  private bindEvents(): void {
    this.ui.yaw.addEventListener("input", event => this.setYaw((event.target as HTMLInputElement).value));
    this.ui.outlineMode.addEventListener("input", () => {
      this.state.outlineMode = this.ui.outlineMode.value as AppState["outlineMode"];
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
    this.ui.buttons.forEach(button => button.addEventListener("click", () => this.setYaw(Number(button.dataset.yaw))));

    this.ui.stage.addEventListener("pointerdown", event => {
      this.state.dragging = true;
      this.state.dragStartX = event.clientX;
      this.state.dragStartYaw = this.state.yaw;
      this.ui.stage.setPointerCapture(event.pointerId);
    });
    this.ui.stage.addEventListener("pointermove", event => {
      if (!this.state.dragging) return;
      const dx = event.clientX - this.state.dragStartX;
      this.setYaw(this.state.dragStartYaw + dx * 0.5);
    });
    this.ui.stage.addEventListener("pointerup", event => {
      this.state.dragging = false;
      this.ui.stage.releasePointerCapture(event.pointerId);
    });
    this.ui.stage.addEventListener("pointercancel", () => {
      this.state.dragging = false;
    });
  }

  private setYaw(value: number | string): void {
    this.state.yaw = ((Number(value) % 360) + 360) % 360;
    this.render();
  }

  private rebuild(): void {
    this.state.samples = Number(this.ui.samples.value);
    this.state.smooth = Number(this.ui.smooth.value);
    this.state.depth = Number(this.ui.depth.value);
    this.state.rigs.clear();
    this.state.earRigs.clear();
    this.render();
  }

  private attachGeometry(
    flatParts: EvaluatedBodyPart[],
    parentById: Map<string, string | null>,
    viewState: ReturnType<typeof computeRigViewState>,
    yaw: number,
    yawRad: number,
  ): Map<string, GeometryPayload> {
    const geometryById = new Map<string, GeometryPayload>();

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

  private render(): void {
    const yaw = ((this.state.yaw % 360) + 360) % 360;
    const rad = yaw * Math.PI / 180;
    const viewContext = computeViewContext(yaw);
    const viewState = computeRigViewState(rad, yaw);
    const policy = computeOutlinePolicy(viewState, this.state.outlineMode);

    const evaluatedTree = evaluateBodyPart(this.bodyTree, viewContext);
    const flatParts = flattenParts(evaluatedTree);
    const parentById = collectParentIds(evaluatedTree);
    this.attachGeometry(flatParts, parentById, viewState, yaw, rad);
    const fusionGroups = computeFusionGroups(flatParts);
    const renderList = computeRenderPasses(flatParts);
    const primitives = generateAnatomyRenderPrimitives(renderList, policy);

    this.primitiveRenderer.render(primitives);

    const lightingSource =
      renderList.find(part => part.source.render?.primaryLighting && part.geometry?.points)?.geometry?.points ?? null;
    const sampleSource =
      renderList.find(part => part.source.render?.debugSamplePoints && part.geometry?.points)?.geometry?.points ?? [];

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
    this.ui.yawOut.textContent = `${Math.round(yaw)}°`;
    this.ui.outlineOut.textContent = this.state.outlineMode;
    this.ui.samplesOut.textContent = `${this.state.samples}`;
    this.ui.smoothOut.textContent = this.state.smooth.toFixed(2);
    this.ui.depthOut.textContent = this.state.depth.toFixed(2);
    this.ui.hudYaw.textContent = `yaw: ${Math.round(yaw)}°`;
    this.ui.hudBlend.textContent = `blend: CENTER ${centerPct}% -> PROFILE ${profilePct}% / BACK ${backPct}%`;
    this.ui.hudMode.textContent = formatHeadMode(viewState);
    this.ui.hudFusion.textContent = formatHeadFusion(viewState, skinFusion);
    this.ui.hudOutline.textContent = `outline: ${this.state.outlineMode}`;
    this.ui.hudOrder.textContent = describePrimitiveOrder(primitives);

    this.ui.stage.classList.toggle("showSources", this.ui.sources.checked);
    this.ui.stage.classList.toggle("showWire", this.ui.wire.checked);
    this.ui.stage.classList.toggle("showDots", this.ui.dots.checked);
    this.ui.buttons.forEach(button => {
      const value = Number(button.dataset.yaw);
      button.classList.toggle("active", Math.abs(value - Math.round(yaw)) < 1);
    });

    renderHeadSources(this.ui, viewState.side, viewContext.back);
  }
}
