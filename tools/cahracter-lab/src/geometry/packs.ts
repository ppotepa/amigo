import { generatedMassGeometryResolverEntries } from "./generated";
import { headDetailGeometryResolverEntries, headMassGeometryResolverEntries } from "./head";
import type { DetailGeometryResolver, MassGeometryResolver } from "./registry";

export type GeometryResolverPack = {
  mass?: Record<string, MassGeometryResolver>;
  detail?: Record<string, DetailGeometryResolver>;
};

export const geometryResolverPacks: GeometryResolverPack[] = [
  {
    mass: headMassGeometryResolverEntries,
    detail: headDetailGeometryResolverEntries,
  },
  {
    mass: generatedMassGeometryResolverEntries,
  },
];
