import { useEffect, useState } from "react";
import { DbInfo, ipc } from "../lib/ipc";

export default function SettingsPage() {
  const [info, setInfo] = useState<DbInfo | null>(null);

  useEffect(() => {
    ipc.dbInfo().then(setInfo);
  }, []);

  async function handleWipe() {
    if (!confirm("Wipe ALL extractions, edits, and documents from the local DB? This cannot be undone.")) return;
    if (!confirm("Are you really sure? (last warning)")) return;
    await ipc.wipeDb();
    setInfo(await ipc.dbInfo());
  }

  return (
    <div>
      <header style={{ marginBottom: 16 }}>
        <h1 style={{ margin: 0, fontSize: 24 }}>Settings</h1>
        <p style={{ margin: "4px 0 0 0", color: "var(--color-fg-muted)" }}>
          Local-only configuration and diagnostics.
        </p>
      </header>

      <section style={section}>
        <h2 style={h2}>Local database</h2>
        {info ? (
          <table style={{ width: "100%" }}>
            <tbody>
              <tr><td style={k}>Path</td><td style={v}><code>{info.path}</code></td></tr>
              <tr><td style={k}>Size</td><td style={v}>{formatBytes(info.size_bytes)}</td></tr>
              <tr><td style={k}>Documents</td><td style={v}>{info.document_count}</td></tr>
              <tr><td style={k}>Extractions</td><td style={v}>{info.extraction_count}</td></tr>
              <tr><td style={k}>Edits</td><td style={v}>{info.edit_count}</td></tr>
            </tbody>
          </table>
        ) : (
          <p>Loading…</p>
        )}
        <div style={{ marginTop: 16, display: "flex", gap: 8 }}>
          <button onClick={() => navigator.clipboard.writeText(info?.path ?? "")}>📋 Copy path</button>
          <button className="danger" onClick={handleWipe}>⚠️ Wipe local DB</button>
        </div>
      </section>

      <section style={section}>
        <h2 style={h2}>Privacy</h2>
        <p style={{ color: "var(--color-fg-muted)" }}>
          All extraction happens on this device. No documents, schemas, or extracted data
          are sent to any server. The model (cactus-needle 3.2.1) and the Python interpreter
          are bundled inside the app.
        </p>
      </section>

      <section style={section}>
        <h2 style={h2}>About</h2>
        <p>RealEstate-Bench v0.1.0 — local document-AI extraction.</p>
        <p>© 2026 Swaroop Kallakuri. All rights reserved.</p>
      </section>
    </div>
  );
}

const section: React.CSSProperties = {
  background: "var(--color-bg-elevated)",
  border: "1px solid var(--color-border)",
  borderRadius: "var(--radius-md)",
  padding: 20,
  marginBottom: 16,
};
const h2: React.CSSProperties = { margin: "0 0 12px 0", fontSize: 16 };
const k: React.CSSProperties = { padding: "4px 0", color: "var(--color-fg-muted)", width: 140 };
const v: React.CSSProperties = { padding: "4px 0" };

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} MB`;
  return `${(n / 1024 / 1024 / 1024).toFixed(2)} GB`;
}
