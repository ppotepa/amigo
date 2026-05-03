import { Brush } from "lucide-react";

export function ThemeButton({ onClick }: { onClick: () => void }) {
  return (
    <button className="button button-ghost" type="button" onClick={onClick}>
      <Brush size={16} />
      Theme
    </button>
  );
}
