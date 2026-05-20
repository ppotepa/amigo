import { NS, type UiRefs } from "./types";
import { injectInlineStyles } from "./render";

export function exportCurrentSvg(ui: UiRefs, rerender: () => void, yaw: number): void {
  rerender();
  const clone = ui.stage.cloneNode(true) as SVGSVGElement;
  clone.setAttribute("xmlns", NS);
  clone.setAttribute("width", "1080");
  clone.setAttribute("height", "1080");
  clone.querySelectorAll(".axisLine").forEach(node => node.remove());
  injectInlineStyles(clone);

  const content = `<?xml version="1.0" encoding="UTF-8"?>\n${new XMLSerializer().serializeToString(clone)}\n`;
  const blob = new Blob([content], { type: "image/svg+xml" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `character-yaw-${Math.round(yaw)}.svg`;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 0);
}
