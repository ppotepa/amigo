import type { EditorComponentProps } from "../../editor-components/componentTypes";

export function RegisteredWindowPanel({ instance }: EditorComponentProps) {
  return (
    <section className="workspace-empty">
      <p>Registered component</p>
      <code>{instance.componentId}</code>
    </section>
  );
}
