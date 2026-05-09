import { useMemo, useState } from "react";
import type { AssetRegistryDto } from "../../api/dto";
import {
  filterPickableAssets,
  pickableAssetsFromRegistry,
} from "./assetPickerModel";

export type AssetPickerProps = {
  open: boolean;
  registry: AssetRegistryDto | null | undefined;
  domain?: string | null;
  query?: string;
  onClose: () => void;
  onPick: (assetKey: string) => void;
};

export function AssetPicker({
  open,
  registry,
  domain,
  query = "",
  onClose,
  onPick,
}: AssetPickerProps) {
  const [localQuery, setLocalQuery] = useState(query);
  const assets = useMemo(
    () => filterPickableAssets(pickableAssetsFromRegistry(registry), localQuery, domain),
    [domain, localQuery, registry],
  );

  if (!open) return null;

  return (
    <div className="context-picker asset-picker" role="dialog" aria-label="Pick asset">
      <header className="context-picker-header">
        <strong>Assign Asset</strong>
        <button type="button" className="context-icon-button" title="Close" onClick={onClose}>
          x
        </button>
      </header>
      <input
        className="context-picker-search"
        type="search"
        value={localQuery}
        placeholder="Filter assets"
        onChange={(event) => setLocalQuery(event.currentTarget.value)}
      />
      <div className="context-picker-list">
        {assets.length ? assets.map((asset) => (
          <button
            key={asset.key}
            type="button"
            className="context-picker-item"
            onClick={() => onPick(asset.key)}
          >
            <span>{asset.label}</span>
            <small>{[asset.domain, asset.kind].filter(Boolean).join(" / ")}</small>
            {asset.path ? <small>{asset.path}</small> : null}
          </button>
        )) : (
          <p className="muted workspace-note">No matching assets.</p>
        )}
      </div>
    </div>
  );
}
