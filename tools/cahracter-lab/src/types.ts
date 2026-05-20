export const NS = "http://www.w3.org/2000/svg";
export const PIVOT_X = 540;
export const PIVOT_Y = 540;

export const OUTLINE_MODES = {
  FULL: "FULL",
  ADAPTIVE: "ADAPTIVE",
  SILHOUETTE_ONLY: "SILHOUETTE_ONLY",
  PAINTERLY: "PAINTERLY",
} as const;

export type OutlineMode = (typeof OUTLINE_MODES)[keyof typeof OUTLINE_MODES];
export type BodyPartType = "group" | "mass" | "detail" | "contour";
export type VisibilityMode = "always" | "front-only" | "front-profile" | "front-side" | "not-back";
export type RenderPass = "farMass" | "midMass" | "nearMass" | "detail" | "outline";
export type GeometryStrategyId = string;

export type Point = {
  x: number;
  y: number;
};

export type Anchor3D = {
  x: number;
  y: number;
  z: number;
};

export type Bounds = {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
  width: number;
  height: number;
};

export type Contour = {
  id: string;
  points: Point[];
  center: Point;
  area: number;
  length: number;
};

export type MorphPair = {
  key: "face" | "nose";
  source: Point[];
  target: Point[];
  back?: Point[];
};

export type EarRig = {
  key: EarKey;
  localSide: 1 | -1;
  front: Point[];
  profileLeft: Point[];
  back: Point[];
  profileRight: Point[];
};

export type EarKey = "earRight" | "earLeft";

export type EarState = {
  key: EarKey;
  localSide: 1 | -1;
  depth: number;
  screenX: number;
  isNear: boolean;
  isFar: boolean;
  frontLike: boolean;
  profileT: number;
  backT: number;
  fusion: number;
};

export type ViewZone =
  | "FRONT"
  | "THREE_QUARTER"
  | "PROFILE"
  | "REAR_TRANSITION"
  | "BACK_PROXY";

export type NoseMode = "SEPARATE" | "MERGING" | "FUSED" | "HIDDEN";

export type ViewState = {
  yawDeg: number;
  side: 1 | -1;
  profile: number;
  back: number;
  zone: ViewZone;
  t: number;
  noseFusion: number;
  noseMode: NoseMode;
  ears: EarState[];
  earRight: EarState;
  earLeft: EarState;
  nearEar: EarState;
  farEar: EarState;
  showNose: boolean;
  showMouth: boolean;
};

export type ViewContext = {
  angle: number;
  theta: number;
  c: number;
  s: number;
  front: number;
  back: number;
  side: number;
  dir: 1 | -1;
  profileT: number;
  backT: number;
};

export type OutlinePartPolicy = {
  drawBody: boolean;
  drawContour: boolean;
  drawInner: boolean;
  drawSilhouette: boolean;
};

export type OutlinePolicy = {
  drawMasterSilhouette: boolean;
  parts: Record<string, OutlinePartPolicy>;
};

export type UiRefs = {
  stage: SVGSVGElement;
  rigLayer: SVGGElement;
  outlineLayer: SVGGElement;
  debugLayer: SVGGElement;
  sourceGuides: SVGGElement;
  yaw: HTMLInputElement;
  outlineMode: HTMLSelectElement;
  samples: HTMLInputElement;
  smooth: HTMLInputElement;
  depth: HTMLInputElement;
  auto: HTMLInputElement;
  sources: HTMLInputElement;
  wire: HTMLInputElement;
  dots: HTMLInputElement;
  exportSvg: HTMLButtonElement;
  yawOut: HTMLOutputElement;
  outlineOut: HTMLOutputElement;
  samplesOut: HTMLOutputElement;
  smoothOut: HTMLOutputElement;
  depthOut: HTMLOutputElement;
  hudYaw: HTMLDivElement;
  hudBlend: HTMLDivElement;
  hudMode: HTMLDivElement;
  hudFusion: HTMLDivElement;
  hudOutline: HTMLDivElement;
  hudOrder: HTMLDivElement;
  buttons: HTMLButtonElement[];
  shadowStop1: SVGStopElement;
  shadowStop2: SVGStopElement;
  shadowStop3: SVGStopElement;
  lightStop1: SVGStopElement;
  lightStop2: SVGStopElement;
  lightStop3: SVGStopElement;
};

export type AppState = {
  yaw: number;
  samples: number;
  smooth: number;
  depth: number;
  outlineMode: OutlineMode;
  auto: boolean;
  rigs: Map<string, MorphPair[]>;
  earRigs: Map<string, Record<EarKey, EarRig>>;
  dots: SVGCircleElement[];
  lastTime: number;
  dragging: boolean;
  dragStartX: number;
  dragStartYaw: number;
};

export type GeometryStateKey = "front" | "profileLeft" | "profileRight" | "back" | "generated";
export type GeometryInput = {
  sourcePartId?: string;
  parent?: boolean;
};

export type GeometryParams = Record<string, number | string | boolean>;

export type BodyPart = {
  id: string;
  type: BodyPartType;
  anchor: Anchor3D;
  geometry?: {
    strategy: GeometryStrategyId;
    side?: "left" | "right";
    input?: GeometryInput;
    params?: GeometryParams;
    states?: Partial<Record<GeometryStateKey, string>>;
  };
  visibilityMode?: VisibilityMode;
  fusionGroup?: string;
  outlineRole?: "outer" | "inner" | "none";
  renderPass?: RenderPass;
  render?: import("./render-primitives").RenderHints;
  children?: BodyPart[];
};

export type GeometryPayload = {
  points?: Point[];
  path?: string;
  bounds?: Bounds;
  opacity?: number;
  visible?: boolean;
  meta?: Record<string, number | string | boolean>;
};

export type EvaluatedBodyPart = {
  id: string;
  type: BodyPartType;
  source: BodyPart;
  projected: {
    x: number;
    y: number;
    depth: number;
  };
  visible: boolean;
  depth: number;
  renderPass: RenderPass;
  fusion: {
    group: string | null;
    t: number;
  };
  geometry: GeometryPayload | null;
  children: EvaluatedBodyPart[];
};
