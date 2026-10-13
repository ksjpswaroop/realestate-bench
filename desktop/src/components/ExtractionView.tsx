import { useState } from "react";
import { ExtractResult } from "../lib/ipc";
import TagPicker from "./TagPicker";

type Props = {
  result: ExtractResult;
  schemaFields: string[];
  onExportExcel: () => void;
  onExportJson: () => void;
  onExportCsv: () => void;
  onReExtract: () => void;
  onEdit: (fieldPath: string, oldVal: unknown, newVal: unknown) => Promise<void>;
  onReset: () => void;
};

export default function ExtractionView({
  result, schemaFields, onExportExcel, onExportJson, onExportCsv, onReExtract, onEdit, onReset,
}: Props) {
  const [editingField, setEditingField] = useState<string | null>(null);
  const [editValue, setEditValue] = useState<string>("");

  const extraction = (result.extraction ?? {}) as Record<string, unknown>;
  const fields = schemaFields.length > 0 ? schemaFields : Object.keys(extraction);

  function startEdit(field: string) {
    setEditingField(field);
    const v = extraction[field];
    setEditValue(v === null || v === undefined ? "" : String(v));
  }

  async function commitEdit(field: string) {
    const oldVal = extraction[field];
    const oldStr = oldVal === null || oldVal === undefined ? "" : String(oldVal);
    if (editValue === oldStr) {
      setEditingField(null);
      return;
    }
    // Try to preserve type
    let newVal: unknown = editValue;
    if (typeof oldVal === "number") {
      const n = Number(editValue);
      if (!Number.isNaN(n)) newVal = n;
    } else if (oldVal === null || oldVal === undefined) {
      newVal = editValue || null;
    }
    await onEdit(field, oldVal, newVal);
    setEditingField(null);
  }

  return (
    <div>
      <div style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 16 }}>
        <button onClick={onReset}>← New document</button>
        <button onClick={onReExtract}>↻ Re-extract</button>
        <span style={{ flex: 1 }} />
        <button onClick={onExportCsv}>📄 CSV</button>
        <button onClick={onExportJson}>📋 JSON</button>
        <button className="primary" onClick={onExportExcel}>📊 Excel</button>
      </div>

      <div
        style={{
          background: "var(--color-bg-elevated)",
          border: "1px solid var(--color-border)",
          borderRadius: "var(--radius-md)",
          padding: 16,
          marginBottom: 16,
          fontSize: 13,
        }}
      >
        <span>model <code>{result.model_version}</code> · schema <code>{result.schema_id}</code> v{result.schema_version} · {result.latency_ms}ms</span>
      </div>

      {result.extraction_id > 0 && (
        <div
          style={{
            background: "var(--color-bg-elevated)",
            border: "1px solid var(--color-border)",
            borderRadius: "var(--radius-md)",
            padding: 16,
            marginBottom: 16,
          }}
        >
          <TagPicker extractionId={result.extraction_id} initialTags={[]} />
        </div>
      )}

      <div
        style={{
          background: "var(--color-bg-elevated)",
          border: "1px solid var(--color-border)",
          borderRadius: "var(--radius-md)",
          padding: 16,
        }}
      >
        {fields.map((field) => {
          const value = extraction[field];
          const conf = result.confidences[field] ?? 0;
          const isEditing = editingField === field;
          const confColor = conf >= 0.7 ? "var(--color-success)" : conf >= 0.4 ? "var(--color-warning)" : "var(--color-danger)";

          return (
            <div
              key={field}
              style={{
                display: "grid",
                gridTemplateColumns: "200px 1fr 80px",
                gap: 12,
                padding: "8px 0",
                borderBottom: "1px solid var(--color-border)",
                alignItems: "center",
              }}
            >
              <div style={{ color: "var(--color-fg-muted)", fontFamily: "var(--font-mono)", fontSize: 13 }}>
                {field}
              </div>
              <div
                onClick={() => !isEditing && startEdit(field)}
                style={{
                  cursor: isEditing ? "text" : "pointer",
                  padding: 4,
                  borderRadius: 4,
                  background: isEditing ? "var(--color-bg-sunken)" : "transparent",
                }}
                title={isEditing ? "" : "Click to edit"}
              >
                {isEditing ? (
                  <input
                    autoFocus
                    value={editValue}
                    onChange={(e) => setEditValue(e.target.value)}
                    onBlur={() => commitEdit(field)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") commitEdit(field);
                      if (e.key === "Escape") setEditingField(null);
                    }}
                    style={{ width: "100%" }}
                  />
                ) : (
                  <span>{formatValue(value)}</span>
                )}
              </div>
              <div style={{ color: confColor, fontSize: 12, fontFamily: "var(--font-mono)", textAlign: "right" }}>
                {(conf * 100).toFixed(0)}%
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function formatValue(v: unknown): React.ReactNode {
  if (v === null || v === undefined) return <em style={{ color: "var(--color-fg-muted)" }}>(empty)</em>;
  if (typeof v === "string") return v;
  return JSON.stringify(v);
}
