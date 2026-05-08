import { componentMenuGroups } from "../editor-components/componentMenu";
import { iconForEditorComponent } from "../editor-components/componentRegistry";
import type { EditorComponentDefinition } from "../editor-components/componentTypes";
import { toneForComponentDomain } from "../theme/semanticColorRegistry";

export function ComponentMenu({ onOpen }: { onOpen: (component: EditorComponentDefinition<any>) => void }) {
  const groups = componentMenuGroups().filter((group) =>
    group.components.some((component) => component.canDock || component.canOpenInWindow || component.canOpenInCenterTabs),
  );

  return (
    <div className="component-menu-popover">
      {groups.map((group) => (
        <section key={group.category}>
          <h3>{group.category}</h3>
          {group.components.map((component) => (
            <button key={component.id} type="button" onClick={() => onOpen(component)}>
              {iconForEditorComponent(component.icon, 13, toneForComponentDomain(component.domain))}
              <span>
                <strong>{component.title}</strong>
                <small>{component.domain}{component.subdomain ? ` · ${component.subdomain}` : ""}</small>
              </span>
            </button>
          ))}
        </section>
      ))}
    </div>
  );
}
