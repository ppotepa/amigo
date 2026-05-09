import { useState } from "react";
import type { AssetRegistryDto } from "../../api/dto";
import { AssetPicker } from "../assets/AssetPicker";

export type AssetRefFieldProps = {
  value: string | null | undefined;
  assetDomain?: string | null;
  required?: boolean;
  registry: AssetRegistryDto | null | undefined;
  onAssign: (assetKey: string | null) => void;
};

export function AssetRefField({
  value,
  assetDomain,
  required = false,
  registry,
  onAssign,
}: AssetRefFieldProps) {
  const [pickerOpen, setPickerOpen] = useState(false);
  const missing = required && !value;

  return (
    <div className="asset-ref-field">
      <span className={missing ? "asset-ref-value missing" : "asset-ref-value"}>
        {value || (missing ? "missing required asset" : "none")}
      </span>
      <div className="asset-ref-actions">
        <button type="button" className="context-action-button" onClick={() => setPickerOpen(true)}>
          Assign
        </button>
        {!required ? (
          <button type="button" className="context-action-button" onClick={() => onAssign(null)}>
            Clear
          </button>
        ) : null}
      </div>
      <AssetPicker
        open={pickerOpen}
        registry={registry}
        domain={assetDomain}
        onClose={() => setPickerOpen(false)}
        onPick={(assetKey) => {
          onAssign(assetKey);
          setPickerOpen(false);
        }}
      />
    </div>
  );
}
