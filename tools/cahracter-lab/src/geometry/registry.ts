import { type AppState, type BodyPart, type GeometryPayload, type ViewState } from "../types";
import { geometryResolverPacks } from "./packs";

export type MassGeometryResolver = (
  part: BodyPart,
  state: AppState,
  viewState: ViewState,
  yaw: number,
  yawRad: number,
) => GeometryPayload | null;

export type DetailGeometryResolver = (
  part: BodyPart,
  parentGeometry: GeometryPayload | null,
  viewState: ViewState,
  outlineMode: string,
) => GeometryPayload | null;

type ResolverEntries<TResolver> = Record<string, TResolver>;

function combineResolverEntries<TResolver>(...packs: ResolverEntries<TResolver>[]): ResolverEntries<TResolver> {
  return Object.assign({}, ...packs);
}

export const massGeometryResolvers: Record<string, MassGeometryResolver> = combineResolverEntries(
  ...geometryResolverPacks.map(pack => pack.mass ?? {}),
);

export const detailGeometryResolvers: Record<string, DetailGeometryResolver> = combineResolverEntries(
  ...geometryResolverPacks.map(pack => pack.detail ?? {}),
);
