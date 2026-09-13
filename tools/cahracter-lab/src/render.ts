import { boundsOf } from "./math";
import { createEl } from "./dom";
import { NS, type Point, type UiRefs } from "./types";

export function ensureDots(ui: UiRefs, dots: SVGCircleElement[], count: number): SVGCircleElement[] {
  while (dots.length < count) {
    const dot = createEl("circle", { class: "dot", r: "3.4" });
    ui.debugLayer.appendChild(dot);
    dots.push(dot);
  }
  return dots;
}

export function setGradientDirection(ui: UiRefs, points: Point[], side: 1 | -1, t: number): void {
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

export function injectInlineStyles(target: SVGSVGElement): void {
  const styleText = Array.from(document.styleSheets)
    .map(sheet => {
      try {
        return Array.from(sheet.cssRules).map(rule => rule.cssText).join("\n");
      } catch {
        return "";
      }
    })
    .filter(Boolean)
    .join("\n");

  if (!styleText) return;
  const style = document.createElementNS(NS, "style");
  style.textContent = styleText;
  target.insertBefore(style, target.firstChild);
}
