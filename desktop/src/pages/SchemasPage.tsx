/**
 * SchemasPage — Phase 2 user-facing schema management.
 *
 * Built-in schemas (lease addendum, invoice, rent roll, medical SOAP) are
 * read-only and listed first. Below them, the user can:
 *   - see all schemas they've created
 *   - create a new schema (modal: SchemaBuilder)
 *   - edit / delete an existing one
 *
 * Non-technical. No code, no terminal, no JSON. They pick from a field-type
 * dropdown, type a label, and a sensible snake_case machine name is
 * auto-generated. They can override the name.
 */
import { useEffect, useState } from "react";
import { ipc, SchemaInfo, UserSchemaRow, UserFieldDef } from "../lib/ipc";
import SchemaBuilder from "../components/SchemaBuilder";

type View = "list" | "new" | { edit: UserSchemaRow };

export default function SchemasPage() {
  const [builtIn, setBuiltIn] = useState<SchemaInfo[]>([]);
  const [userSchemas, setUserSchemas] = useState<UserSchemaRow[]>([]);
  const [view, setView] = useState<View>("list");
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    try {
      const [bi, us] = await Promise.all([ipc.listSchemas(), ipc.listUserSchemas()]);
      setBuiltIn(bi.filter((s) => s.version !== "user"));
      setUserSchemas(us);
    } catch (e) {
      setError(`${e}`);
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  const onDelete = async (id: string, name: string) => {
    const ok = window.confirm(
      `Delete schema "${name}"? Existing extractions will still work — they store the schema definition that was used at the time.`,
    );
    if (!ok) return;
    try {
      await ipc.deleteUserSchema(id);
      await refresh();
    } catch (e) {
      setError(`${e}`);
    }
  };

  return (
    <div className="page schemas-page">
      <header className="page-header">
        <h1>Schemas</h1>
        <p className="page-subtitle">
          Built-in schemas handle the common cases. Create your own for
          anything else.
        </p>
      </header>

      {error && (
        <div className="alert error">
          {error}
          <button onClick={() => setError(null)} className="btn-link">×</button>
        </div>
      )}

      {view === "list" && (
        <>
          <section className="schema-section">
            <h2>Built-in schemas</h2>
            <p className="hint">These are versioned and not editable.</p>
            <ul className="schema-list">
              {builtIn.map((s) => (
                <li key={s.id} className="schema-card built-in">
                  <div className="schema-card-header">
                    <h3>{s.name}</h3>
                    <span className="version-tag">v{s.version}</span>
                  </div>
                  {s.doc && <p className="schema-doc">{s.doc}</p>}
                  <details>
                    <summary>{s.fields.length} fields</summary>
                    <ul className="field-chips">
                      {s.fields.map((f) => (
                        <li key={f} className="chip">{f}</li>
                      ))}
                    </ul>
                  </details>
                </li>
              ))}
            </ul>
          </section>

          <section className="schema-section">
            <div className="section-header">
              <h2>Your schemas</h2>
              <button
                className="btn primary"
                onClick={() => setView("new")}
                data-testid="new-schema-button"
              >
                + New schema
              </button>
            </div>
            {userSchemas.length === 0 ? (
              <p className="empty">
                No custom schemas yet. Click <strong>New schema</strong> to
                create one.
              </p>
            ) : (
              <ul className="schema-list">
                {userSchemas.map((s) => {
                  const fields: UserFieldDef[] = (() => {
                    try {
                      return JSON.parse(s.fields_json);
                    } catch {
                      return [];
                    }
                  })();
                  return (
                    <li key={s.id} className="schema-card user">
                      <div className="schema-card-header">
                        <h3>{s.name}</h3>
                        <div className="actions">
                          <button
                            className="btn-link"
                            onClick={() => setView({ edit: s })}
                          >
                            Edit
                          </button>
                          <button
                            className="btn-link danger"
                            onClick={() => onDelete(s.id, s.name)}
                          >
                            Delete
                          </button>
                        </div>
                      </div>
                      {s.description && <p className="schema-doc">{s.description}</p>}
                      <details>
                        <summary>{fields.length} fields</summary>
                        <ul className="field-chips">
                          {fields.map((f) => (
                            <li key={f.name} className="chip">
                              {f.label}
                              <span className="chip-type">{f.type}</span>
                            </li>
                          ))}
                        </ul>
                      </details>
                    </li>
                  );
                })}
              </ul>
            )}
          </section>
        </>
      )}

      {view === "new" && (
        <SchemaBuilder
          onCancel={() => setView("list")}
          onSaved={async () => {
            setView("list");
            await refresh();
          }}
          onError={setError}
        />
      )}

      {typeof view === "object" && "edit" in view && (
        <SchemaBuilder
          existing={view.edit}
          onCancel={() => setView("list")}
          onSaved={async () => {
            setView("list");
            await refresh();
          }}
          onError={setError}
        />
      )}
    </div>
  );
}
