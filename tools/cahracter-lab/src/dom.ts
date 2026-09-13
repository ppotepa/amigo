import { NS, type UiRefs } from "./types";

function requireElement<T extends Element>(id: string): T {
  const element = document.getElementById(id);
  if (!element) throw new Error(`Missing element #${id}`);
  return element as unknown as T;
}

export function createEl<K extends keyof SVGElementTagNameMap>(
  name: K,
  attrs: Record<string, string> = {},
): SVGElementTagNameMap[K] {
  const node = document.createElementNS(NS, name);
  for (const [key, value] of Object.entries(attrs)) node.setAttribute(key, value);
  return node;
}

export function getUiRefs(): UiRefs {
  return {
    stage: requireElement<SVGSVGElement>("stage"),
    rigLayer: requireElement<SVGGElement>("rigLayer"),
    outlineLayer: requireElement<SVGGElement>("outlineLayer"),
    debugLayer: requireElement<SVGGElement>("debugLayer"),
    sourceGuides: requireElement<SVGGElement>("sourceGuides"),
    yaw: requireElement<HTMLInputElement>("yaw"),
    outlineMode: requireElement<HTMLSelectElement>("outlineMode"),
    samples: requireElement<HTMLInputElement>("samples"),
    smooth: requireElement<HTMLInputElement>("smooth"),
    depth: requireElement<HTMLInputElement>("depth"),
    auto: requireElement<HTMLInputElement>("auto"),
    sources: requireElement<HTMLInputElement>("sources"),
    wire: requireElement<HTMLInputElement>("wire"),
    dots: requireElement<HTMLInputElement>("dots"),
    exportSvg: requireElement<HTMLButtonElement>("exportSvg"),
    yawOut: requireElement<HTMLOutputElement>("yawOut"),
    outlineOut: requireElement<HTMLOutputElement>("outlineOut"),
    samplesOut: requireElement<HTMLOutputElement>("samplesOut"),
    smoothOut: requireElement<HTMLOutputElement>("smoothOut"),
    depthOut: requireElement<HTMLOutputElement>("depthOut"),
    hudYaw: requireElement<HTMLDivElement>("hudYaw"),
    hudBlend: requireElement<HTMLDivElement>("hudBlend"),
    hudMode: requireElement<HTMLDivElement>("hudMode"),
    hudFusion: requireElement<HTMLDivElement>("hudFusion"),
    hudOutline: requireElement<HTMLDivElement>("hudOutline"),
    hudOrder: requireElement<HTMLDivElement>("hudOrder"),
    buttons: Array.from(document.querySelectorAll<HTMLButtonElement>("[data-yaw]")),
    shadowStop1: requireElement<SVGStopElement>("shadowStop1"),
    shadowStop2: requireElement<SVGStopElement>("shadowStop2"),
    shadowStop3: requireElement<SVGStopElement>("shadowStop3"),
    lightStop1: requireElement<SVGStopElement>("lightStop1"),
    lightStop2: requireElement<SVGStopElement>("lightStop2"),
    lightStop3: requireElement<SVGStopElement>("lightStop3"),
  };
}
