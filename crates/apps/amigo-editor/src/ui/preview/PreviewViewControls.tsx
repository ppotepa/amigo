export type PreviewZoomOption = number;

export function PreviewViewControls({
  label = "Zoom",
  onZoomChange,
  zoom,
  zoomOptions,
}: {
  label?: string;
  onZoomChange: (zoom: number) => void;
  zoom: number;
  zoomOptions: readonly PreviewZoomOption[];
}) {
  return (
    <label className="preview-view-controls">
      <span>{label}</span>
      <select
        className="preview-zoom-select"
        value={zoom}
        onChange={(event) => onZoomChange(Number(event.target.value))}
      >
        {zoomOptions.map((option) => (
          <option key={option} value={option}>
            {Math.round(option * 100)}%
          </option>
        ))}
      </select>
    </label>
  );
}
