import { remapClamp, smoothstep } from "./math";
import { createEl } from "./dom";
import { type UiRefs, type ViewState } from "./types";

export function renderHeadSources(ui: UiRefs, side: 1 | -1, backT = 0): void {
  ui.sourceGuides.innerHTML = "";
  const centerSelectors = ["#CENTER #FACE", "#CENTER #NOSE", side > 0 ? "#CENTER #PRAWE" : "#CENTER #LEWE"];
  const profileSelectors = ["#LEFT_PROFILE #FACE_PROFILE", "#LEFT_PROFILE #NOSE-PROFILE", "#LEFT_PROFILE #EAR_LEFT"];

  for (const selector of centerSelectors) {
    const path = document.querySelector<SVGPathElement>(selector);
    if (!path) continue;
    ui.sourceGuides.appendChild(createEl("path", { class: "ghostPath centerGhost", d: path.getAttribute("d") ?? "" }));
  }

  for (const selector of profileSelectors) {
    const path = document.querySelector<SVGPathElement>(selector);
    if (!path) continue;
    const ghost = createEl("path", { class: "ghostPath profileGhost", d: path.getAttribute("d") ?? "" });
    if (side < 0) ghost.setAttribute("transform", "translate(1080,0) scale(-1,1)");
    ui.sourceGuides.appendChild(ghost);
  }

  ui.sourceGuides.style.opacity = String(1 - smoothstep(remapClamp(backT, 0.45, 1.0)) * 0.58);
}

export function formatHeadMode(viewState: ViewState): string {
  const sideName = viewState.side > 0 ? "LEFT" : "RIGHT";
  if (viewState.zone === "FRONT") return `zone: FRONT / nose ${viewState.noseMode}`;
  if (viewState.zone === "BACK_PROXY") return "zone: BACK_PROXY / nose+mouth state-hidden";
  return `zone: ${viewState.zone} ${sideName} / nose ${viewState.noseMode}`;
}

export function formatHeadFusion(viewState: ViewState, skinFusionCount: number): string {
  return `fusion: nose ${viewState.noseFusion.toFixed(2)} / R ${viewState.earRight.fusion.toFixed(2)} / L ${viewState.earLeft.fusion.toFixed(2)} / skin ${skinFusionCount}`;
}
