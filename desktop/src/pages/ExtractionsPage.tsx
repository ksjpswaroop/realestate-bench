import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { ipc, ExtractionRow } from "../lib/ipc";

export default function ExtractionsPage() {
  const [rows, setRows] = useState<ExtractionRow[]>([]);
  const [search, setSearch] = useState("");
  const [schemaFilter, setSchemaFilter] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [selected, setSelected] = useState<Set<number>>(new Set());

  useEffect(() => {
    refresh();
  }, [schemaFilter]);

  async function refresh() {
    setLoading(true);
    try {
      const r = await ipc.listExtractions(schemaFilter || undefined, search || undefined);
      setRows(r);
    } finally {
      setLoading(false);
    }
  }

  const schemas = useMemo(
    () => Array.from(new Set(rows.map((r) => r.schema_id))),
    [rows]
  );

  function toggle(id: number) {
    const s = new Set(selected);
    if (s.has(id)) s.delete(id);
    else s.add(id);
    setSelected(s);
  }

  async function handleDelete(id: number) {
    if (!confirm("Delete this extraction? (soft delete; can be recovered from DB)")) return;
    await ipc.deleteExtraction(id);
    setRows(rows.filter((r) => r.id !== id));
  }

  return (
    <div>
      <header style={{ marginBottom: 16 }}>
        <h1 style={{ margin: 0, fontSize: 24 }}>Extractions</h1>
        <p style={{ margin: "4px 0 0 0", color: "var(--color-fg-muted)" }}>
          {rows.length} extraction{rows.length === 1 ? "" : "s"} in local DB.
        </p>
      </header>

      <div style={{ display: "flex", gap: 12, marginBottom: 16, alignItems: "center" }}>
        <input
          placeholder="Search across all extractions…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter") refresh(); }}
          style={{ flex: 1 }}
        />
        <button onClick={refresh}>🔍 Search</button>
        <select
          value={schemaFilter}
          onChange={(e) => setSchemaFilter(e.target.value)}
          style={{ width: 200 }}
        >
          <option value="">All schemas</option>
          {schemas.map((s) => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
      </div>

      {loading ? (
        <p>Loading…</p>
      ) : rows.length === 0 ? (
        <p style={{ color: "var(--color-fg-muted)" }}>No extractions yet. Drop a document on the Extract tab.</p>
      ) : (
        <table style={{ width: "100%", borderCollapse: "collapse" }}>
          <thead>
            <tr style={{ textAlign: "left", borderBottom: "1px solid var(--color-border)" }}>
              <th style={th}><input type="checkbox" disabled /></th>
              <th style={th}>Document</th>
              <th style={th}>Schema</th>
              <th style={th}>Confidence</th>
              <th style={th}>Created</th>
              <th style={th}>Latency</th>
              <th style={th}></th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.id} style={{ borderBottom: "1px solid var(--color-border)" }}>
                <td style={td}>
                  <input
                    type="checkbox"
                    checked={selected.has(r.id)}
                    onChange={() => toggle(r.id)}
                  />
                </td>
                <td style={td}>
                  <Link to={`/extractions/${r.id}`} style={{ color: "var(--color-accent)" }}>
                    {r.document_name}
                  </Link>
                </td>
                <td style={td}><code>{r.schema_id}</code></td>
                <td style={td}>
                  <Confidence value={r.mean_confidence} />
                </td>
                <td style={td}>{new Date(r.created_at).toLocaleString()}</td>
                <td style={td}>{r.latency_ms}ms</td>
                <td style={td}>
                  <button className="danger" onClick={() => handleDelete(r.id)}>🗑</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

const th: React.CSSProperties = { padding: "8px 6px", fontWeight: 600, fontSize: 12, color: "var(--color-fg-muted)" };
const td: React.CSSProperties = { padding: "8px 6px" };

function Confidence({ value }: { value: number }) {
  const color = value >= 0.7 ? "var(--color-success)" : value >= 0.4 ? "var(--color-warning)" : "var(--color-danger)";
  return (
    <span style={{ color, fontWeight: 600 }}>
      {(value * 100).toFixed(0)}%
    </span>
  );
}
