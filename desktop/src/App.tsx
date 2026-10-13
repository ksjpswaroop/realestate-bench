import { NavLink, Outlet } from "react-router-dom";
import { useEffect, useState } from "react";
import { ipc, SchemaInfo } from "./lib/ipc";

export default function App() {
  const [schemas, setSchemas] = useState<SchemaInfo[]>([]);
  const [appName] = useState("RealEstate-Bench");

  useEffect(() => {
    ipc.listSchemas().then(setSchemas).catch((e) => console.error("listSchemas", e));
  }, []);

  return (
    <div style={{ display: "grid", gridTemplateColumns: "220px 1fr", height: "100%" }}>
      <aside
        style={{
          background: "var(--color-bg-elevated)",
          borderRight: "1px solid var(--color-border)",
          padding: 16,
          display: "flex",
          flexDirection: "column",
        }}
      >
        <div style={{ fontWeight: 600, fontSize: 18, marginBottom: 4 }}>{appName}</div>
        <div style={{ color: "var(--color-fg-muted)", fontSize: 12, marginBottom: 24 }}>
          local document-AI
        </div>

        <NavLink
          to="/"
          end
          style={({ isActive }) => navStyle(isActive)}
        >
          📄 Extract
        </NavLink>
        <NavLink
          to="/extractions"
          style={({ isActive }) => navStyle(isActive)}
        >
          🗂️ Extractions
        </NavLink>
        <NavLink
          to="/schemas"
          style={({ isActive }) => navStyle(isActive)}
        >
          🧩 Schemas
        </NavLink>
        <NavLink
          to="/settings"
          style={({ isActive }) => navStyle(isActive)}
        >
          ⚙️ Settings
        </NavLink>

        <div style={{ marginTop: 32, color: "var(--color-fg-muted)", fontSize: 11, textTransform: "uppercase", letterSpacing: 0.5 }}>
          Schemas
        </div>
        <ul style={{ listStyle: "none", padding: 0, margin: "8px 0 0 0", fontSize: 12 }}>
          {schemas.map((s) => (
            <li key={s.id} style={{ padding: "2px 0", color: "var(--color-fg-muted)" }}>
              {s.id} {s.version === "user" && <span title="user-defined">(you)</span>}
            </li>
          ))}
        </ul>
      </aside>

      <main style={{ overflow: "auto", padding: 24 }}>
        <Outlet />
      </main>
    </div>
  );
}

function navStyle(active: boolean): React.CSSProperties {
  return {
    display: "block",
    padding: "8px 10px",
    borderRadius: "var(--radius-sm)",
    color: active ? "var(--color-accent)" : "var(--color-fg)",
    background: active ? "var(--color-bg-sunken)" : "transparent",
    textDecoration: "none",
    fontWeight: active ? 600 : 400,
    marginBottom: 2,
  };
}
