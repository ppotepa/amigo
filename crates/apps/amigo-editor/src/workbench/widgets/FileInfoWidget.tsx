import { WidgetFrame } from "./WidgetFrame";
import type { FileInfoWidgetModel } from "./widgetTypes";

type FileInfoWidgetProps = {
  model: FileInfoWidgetModel;
  onOpen?: () => void;
  onReveal?: () => void;
};

export function FileInfoWidget({ model, onOpen, onReveal }: FileInfoWidgetProps) {
  return (
    <WidgetFrame id="file-info-widget" title="File Info" compact>
      <dl className="workbench-dl">
        <div>
          <dt>Path</dt>
          <dd>{model.path ?? "—"}</dd>
        </div>
        <div>
          <dt>Script</dt>
          <dd>{model.scriptPath ?? "—"}</dd>
        </div>
        <div>
          <dt>Dirty</dt>
          <dd>{model.dirty ? "Yes" : "No"}</dd>
        </div>
        <div>
          <dt>Type</dt>
          <dd>{model.type ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Modified</dt>
          <dd>{model.modifiedAt ?? "—"}</dd>
        </div>
      </dl>
      <div className="workbench-widget-actions-row">
        {onOpen ? (
          <button type="button" className="workbench-button" onClick={onOpen}>
            Open YAML
          </button>
        ) : null}
        {onReveal ? (
          <button type="button" className="workbench-button" onClick={onReveal}>
            Reveal File
          </button>
        ) : null}
      </div>
    </WidgetFrame>
  );
}
