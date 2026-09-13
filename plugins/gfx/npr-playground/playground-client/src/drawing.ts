import type { BrushDefinition, BrushLibrary, GeometrySource, NprStyleLayer, StrokeTool, Vec4 } from './contracts';

export const sourceNames: Record<GeometrySource, string> = {
  silhouette: 'Kontur zewnętrzny', creases: 'Ostre krawędzie', 'form-lines': 'Linie formy',
  'shadow-hatch': 'Kreskowanie cieni', construction: 'Linie konstrukcyjne',
  wash: 'Podmalówka', 'flat-fill': 'Wypełnienie', paper: 'Papier',
};
export const toolNames: Record<StrokeTool, string> = { pencil: 'Ołówek', fineliner: 'Cienkopis', nib: 'Pióro', brush: 'Pędzel' };
export const title = (value: string) => value.split(/[-_]/).map((word) => word[0]?.toUpperCase() + word.slice(1)).join(' ');
export const isPaint = (source: GeometrySource) => source === 'wash' || source === 'flat-fill';
export const category = (source: GeometrySource) => source === 'paper' ? 'Papier' : isPaint(source) ? 'Farby' : 'Linie';
export function brushFor(layer: NprStyleLayer, library: BrushLibrary): BrushDefinition | undefined {
  const reference = layer.brush?.brush;
  return reference ? library.brushes[reference.id]?.find((brush) => brush.version === reference.version) : undefined;
}
export function latestAppearances(library: BrushLibrary, source: GeometrySource): BrushDefinition[] {
  if (source === 'paper') return [];
  const application = isPaint(source) ? 'surface' : 'stroke';
  return Object.values(library.brushes).map((versions) => [...versions].sort((a,b) => b.version-a.version)[0])
    .filter((brush): brush is BrushDefinition => !!brush && brush.applications.includes(application))
    .sort((a,b) => a.name.localeCompare(b.name));
}
export function effectiveTool(layer: NprStyleLayer, brush?: BrushDefinition): StrokeTool {
  return layer.tool ?? brush?.tool ?? (brush?.medium === 'graphite' || brush?.medium === 'hatching' ? 'pencil' : 'fineliner');
}
export function appearanceLabel(layer: NprStyleLayer, library: BrushLibrary): string {
  if (layer.source === 'paper') return 'Podłoże rysunku';
  if (isPaint(layer.source)) return layer.source === 'wash' ? 'Farba transparentna' : 'Kolor powierzchni';
  return toolNames[effectiveTool(layer, brushFor(layer, library))];
}
export function useAppearance(layer: NprStyleLayer, brush: BrushDefinition): NprStyleLayer {
  return { ...layer, tool: isPaint(layer.source) ? undefined : brush.tool,
    paint: isPaint(layer.source) ? brush.paint : undefined,
    brush: { brush: { id: brush.id, version: brush.version } } };
}
export const colorHex = (color: Vec4) => '#' + color.slice(0,3).map((v) => Math.round(v*255).toString(16).padStart(2,'0')).join('');
export const hexColor = (hex: string): Vec4 => [parseInt(hex.slice(1,3),16)/255, parseInt(hex.slice(3,5),16)/255, parseInt(hex.slice(5,7),16)/255, 1];

export function restoredPanelWidth(key: string, fallback: number, min: number, max: number,
  storage: Pick<Storage,'getItem'> | undefined = typeof localStorage === 'undefined' ? undefined : localStorage): number {
  try {
    const raw = storage?.getItem(key);
    const value = raw == null ? fallback : Number(raw);
    return Number.isFinite(value) ? Math.min(max,Math.max(min,value)) : fallback;
  } catch { return fallback; }
}
