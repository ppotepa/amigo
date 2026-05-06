import { Search } from "lucide-react";

export function ContextSearch({
  onChange,
  placeholder,
  value,
}: {
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
}) {
  return (
    <label className="context-search">
      <Search size={13} />
      <input
        type="search"
        spellCheck={false}
        value={value}
        placeholder={placeholder}
        onChange={(event) => onChange(event.target.value)}
      />
    </label>
  );
}
