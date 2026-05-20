import { createEl } from "./dom";
import { type UiRefs } from "./types";
import { type PrimitiveLayer, type PrimitiveKind, type RenderPrimitive, sortRenderPrimitives } from "./render-primitives";

function layerFor(ui: UiRefs, layer: PrimitiveLayer): SVGElement {
  if (layer === "outline") return ui.outlineLayer;
  if (layer === "debug") return ui.debugLayer;
  return ui.rigLayer;
}

function createPrimitiveElement(primitive: RenderPrimitive): SVGElement {
  return createEl(primitive.kind as keyof SVGElementTagNameMap);
}

function applyPrimitive(element: SVGElement, primitive: RenderPrimitive): void {
  element.setAttribute("data-primitive-id", primitive.id);
  if (primitive.className) element.setAttribute("class", primitive.className);
  else element.removeAttribute("class");

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

export class PrimitiveSvgRenderer {
  private readonly elements = new Map<string, SVGElement>();

  constructor(private readonly ui: UiRefs) {}

  render(primitives: RenderPrimitive[]): void {
    const sorted = sortRenderPrimitives(primitives);
    const live = new Set(sorted.map(item => item.id));

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
      layerFor(this.ui, primitive.layer).appendChild(element);
    }
  }
}
