import { Fragment, type ReactNode } from "react";

export type KeyValueRow = {
  label: string;
  value: ReactNode;
  title?: string;
  visible?: boolean;
};

export function KeyValueSection({
  rows,
  title,
}: {
  rows: KeyValueRow[];
  title: string;
}) {
  const visibleRows = rows.filter((row) => row.visible ?? true);

  return (
    <section className="workspace-section">
      <h3>{title}</h3>
      <dl className="kv-list">
        {visibleRows.map((row) => (
          <Fragment key={row.label}>
            <dt>{row.label}</dt>
            <dd title={row.title}>{row.value}</dd>
          </Fragment>
        ))}
      </dl>
    </section>
  );
}
