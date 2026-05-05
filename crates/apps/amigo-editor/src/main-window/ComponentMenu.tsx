import { componentMenuGroups } from "../editor-components/componentMenu";
import { iconForEditorComponent } from "../editor-components/componentRegistry";

export function ComponentMenu({ onOpen }: { onOpen: (componentId: string) => void }) {
  const groups = componentMenuGroups().filter((group) =>
    group.components.some((component) => component.canDock || component.canOpenInWindow || component.canOpenInCenterTabs),
  );

  return (
    <div className="component-menu-popover">
      {groups.map((group) => (
        <section key={group.category}>
          <h3>{group.category}</h3>
          {group.components.map((component) => (
            <button key={component.id} type="button" onClick={() => onOpen(component.id)}>
              {iconForEditorComponent(component.icon, 13)}
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
