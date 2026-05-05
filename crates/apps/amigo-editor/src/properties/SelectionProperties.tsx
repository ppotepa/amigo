import { PROPERTY_RENDERERS } from "./propertiesRegistry";
import type { EditorSelection, PropertiesContext } from "./propertiesTypes";

export function SelectionProperties({
  context,
  selection,
}: {
  context: PropertiesContext;
  selection: EditorSelection;
}) {
  const renderer = PROPERTY_RENDERERS.find((candidate) => candidate.canRender(selection));
  return <>{renderer?.render(selection, context) ?? null}</>;
}
