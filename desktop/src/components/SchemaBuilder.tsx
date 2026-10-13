/**
 * SchemaBuilder — modal-style form for creating / editing a user schema.
 *
 * The user picks a field type from a dropdown; we render the right
 * secondary inputs (options for Choice, min/max for Number). Machine
 * names (snake_case) are auto-generated from the label, with override.
 *
 * Validation is client-side; the Rust side also validates (defense in
 * depth) and returns precise error messages that we surface.
 */
import { useState } from "react";
import { ipc, UserFieldDef, UserFieldType, UserSchemaRow } from "../lib/ipc";

const FIELD_TYPES: { value: UserFieldType; label: string; hint: string }[] = [
  { value: "short_text", label: "Short text", hint: "Names, IDs, addresses" },
  { value: "long_text",  label: "Long text",  hint: "Notes, descriptions" },
  { value: "number",     label: "Number",     hint: "Amounts, counts, sqft" },
  { value: "date",       label: "Date",       hint: "ISO-8601 (YYYY-MM-DD)" },
  { value: "yes_no",     label: "Yes / No",   hint: "Boolean flag" },
  { value: "choice",     label: "Choice",     hint: "One of a fixed set" },
  { value: "list",       label: "List of items", hint: "Multiple values" },
];

function labelToName(label: string): string {
  return label
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .slice(0, 64) || "field";
}

function emptyField(): UserFieldDef {
  return {
    name: "",
    label: "",
    type: "short_text",
    description: "",
    required: false,
    options: [],
    min: null,
    max: null,
  };
}

type Props = {
  existing?: UserSchemaRow;
  onCancel: () => void;
  onSaved: () => void | Promise<void>;
  onError: (msg: string) => void;
};

export default function SchemaBuilder({ existing, onCancel, onSaved, onError }: Props) {
  const initial: { name: string; description: string; fields: UserFieldDef[] } = existing
    ? {
        name: existing.name,
        description: existing.description,
        fields: (() => {
          try {
            return JSON.parse(existing.fields_json) as UserFieldDef[];
          } catch {
            return [];
          }
        })(),
      }
    : { name: "", description: "", fields: [emptyField()] };

  const [name, setName] = useState(initial.name);
  const [description, setDescription] = useState(initial.description);
  const [fields, setFields] = useState<UserFieldDef[]>(initial.fields);
  const [saving, setSaving] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);

  const updateField = (i: number, patch: Partial<UserFieldDef>) => {
    setFields((cur) =>
      cur.map((f, idx) => {
        if (idx !== i) return f;
        const merged = { ...f, ...patch };
        // Auto-generate machine name from label when name is empty or matches
        // the previous auto-derived value.
        if (patch.label !== undefined && (f.name === "" || f.name === labelToName(f.label))) {
          merged.name = labelToName(patch.label);
        }
        return merged;
      }),
    );
  };

  const addField = () => setFields((cur) => [...cur, emptyField()]);
  const removeField = (i: number) =>
    setFields((cur) => cur.filter((_, idx) => idx !== i));
  const moveField = (i: number, dir: -1 | 1) => {
    setFields((cur) => {
      const j = i + dir;
      if (j < 0 || j >= cur.length) return cur;
      const next = [...cur];
      [next[i], next[j]] = [next[j], next[i]];
      return next;
    });
  };

  const clientValidate = (): string | null => {
    if (!name.trim()) return "Schema name is required.";
    if (fields.length === 0) return "Add at least one field.";
    const seen = new Set<string>();
    for (let i = 0; i < fields.length; i++) {
      const f = fields[i];
      if (!f.label.trim()) return `Field #${i + 1}: label is required.`;
      if (!f.name.trim()) return `Field #${i + 1}: machine name is required.`;
      if (!/^[a-z0-9_]+$/.test(f.name))
        return `Field #${i + 1}: machine name must be snake_case (a-z, 0-9, underscore).`;
      if (seen.has(f.name))
        return `Field #${i + 1}: duplicate machine name "${f.name}".`;
      seen.add(f.name);
      if (f.type === "choice" && f.options.length === 0)
        return `Field "${f.label}": Choice fields need at least one option.`;
    }
    return null;
  };

  const onSave = async () => {
    const v = clientValidate();
    if (v) {
      setValidationError(v);
      return;
    }
    setValidationError(null);
    setSaving(true);
    try {
      // Strip empty options for non-choice types
      const cleaned = fields.map((f) => ({
        ...f,
        options: f.type === "choice" ? f.options : [],
        min: f.type === "number" ? f.min : null,
        max: f.type === "number" ? f.max : null,
      }));
      if (existing) {
        await ipc.updateUserSchema(existing.id, name, description, cleaned);
      } else {
        await ipc.createUserSchema(name, description, cleaned);
      }
      await onSaved();
    } catch (e) {
      onError(`${e}`);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="schema-builder">
      <header className="page-header">
        <h1>{existing ? "Edit schema" : "New schema"}</h1>
      </header>

      <div className="form-row">
        <label htmlFor="schema-name">Schema name</label>
        <input
          id="schema-name"
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="Vendor Invoice, Inspection Report, ..."
          maxLength={80}
          data-testid="schema-name-input"
        />
      </div>

      <div className="form-row">
        <label htmlFor="schema-desc">
          Description
          <span className="hint"> — instructions for the AI on what to look for</span>
        </label>
        <textarea
          id="schema-desc"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Extract line items, totals, and terms from a vendor invoice."
          rows={3}
          maxLength={2000}
        />
      </div>

      <h2>Fields</h2>
      {validationError && (
        <div className="alert error">{validationError}</div>
      )}

      <ol className="field-editor-list">
        {fields.map((f, i) => (
          <li key={i} className="field-editor">
            <div className="field-editor-head">
              <span className="field-index">{i + 1}.</span>
              <input
                type="text"
                value={f.label}
                onChange={(e) => updateField(i, { label: e.target.value })}
                placeholder="Field label (e.g. Vendor Name)"
                className="field-label-input"
                data-testid={`field-${i}-label`}
              />
              <select
                value={f.type}
                onChange={(e) => updateField(i, { type: e.target.value as UserFieldType })}
                className="field-type-select"
              >
                {FIELD_TYPES.map((t) => (
                  <option key={t.value} value={t.value} title={t.hint}>
                    {t.label}
                  </option>
                ))}
              </select>
              <label className="checkbox-inline">
                <input
                  type="checkbox"
                  checked={f.required}
                  onChange={(e) => updateField(i, { required: e.target.checked })}
                />
                Required
              </label>
              <div className="field-actions">
                <button
                  type="button"
                  className="btn-icon"
                  onClick={() => moveField(i, -1)}
                  disabled={i === 0}
                  aria-label="Move up"
                >↑</button>
                <button
                  type="button"
                  className="btn-icon"
                  onClick={() => moveField(i, 1)}
                  disabled={i === fields.length - 1}
                  aria-label="Move down"
                >↓</button>
                <button
                  type="button"
                  className="btn-icon danger"
                  onClick={() => removeField(i)}
                  disabled={fields.length === 1}
                  aria-label="Remove"
                >×</button>
              </div>
            </div>
            <div className="field-editor-body">
              <input
                type="text"
                value={f.name}
                onChange={(e) => updateField(i, { name: e.target.value.replace(/[^a-z0-9_]/g, "_") })}
                placeholder="machine_name"
                className="field-name-input"
              />
              <textarea
                value={f.description}
                onChange={(e) => updateField(i, { description: e.target.value })}
                placeholder="What should the AI extract into this field?"
                rows={1}
                maxLength={500}
                className="field-desc-input"
              />
            </div>
            {f.type === "choice" && (
              <div className="field-editor-extra">
                <label>Options (one per line):</label>
                <textarea
                  value={f.options.join("\n")}
                  onChange={(e) =>
                    updateField(i, {
                      options: e.target.value
                        .split("\n")
                        .map((s) => s.trim())
                        .filter(Boolean),
                    })
                  }
                  rows={3}
                  placeholder={"Pending\nPaid\nOverdue\nDisputed"}
                />
              </div>
            )}
            {f.type === "number" && (
              <div className="field-editor-extra">
                <label>
                  Min:{" "}
                  <input
                    type="number"
                    value={f.min ?? ""}
                    onChange={(e) =>
                      updateField(i, { min: e.target.value === "" ? null : Number(e.target.value) })
                    }
                    style={{ width: 100 }}
                  />
                </label>
                <label style={{ marginLeft: 16 }}>
                  Max:{" "}
                  <input
                    type="number"
                    value={f.max ?? ""}
                    onChange={(e) =>
                      updateField(i, { max: e.target.value === "" ? null : Number(e.target.value) })
                    }
                    style={{ width: 100 }}
                  />
                </label>
              </div>
            )}
          </li>
        ))}
      </ol>

      <button type="button" className="btn-link" onClick={addField}>
        + Add field
      </button>

      <footer className="form-footer">
        <button className="btn" onClick={onCancel} disabled={saving}>
          Cancel
        </button>
        <button
          className="btn primary"
          onClick={onSave}
          disabled={saving}
          data-testid="save-schema-button"
        >
          {saving ? "Saving…" : existing ? "Save changes" : "Create schema"}
        </button>
      </footer>
    </div>
  );
}
