import { useEffect, useState, useCallback } from "react";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { useParams } from "react-router-dom";
import { ipc, ExtractResult, SchemaInfo } from "../lib/ipc";
import ExtractionView from "../components/ExtractionView";

export default function DropPage() {
  const { id: routeId } = useParams<{ id?: string }>();
  const [schemas, setSchemas] = useState<SchemaInfo[]>([]);
  const [schemaId, setSchemaId] = useState<string>("lease_addendum");
  const [extracting, setExtracting] = useState(false);
  const [result, setResult] = useState<ExtractResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const [currentFile, setCurrentFile] = useState<string | null>(null);

  useEffect(() => {
    ipc.listSchemas().then(setSchemas).catch(console.error);
  }, []);

  // Load a specific extraction if we navigated to /extractions/:id
  useEffect(() => {
    if (!routeId) return;
    ipc.getExtraction(Number(routeId)).then((row) => {
      const extraction = JSON.parse(row.extraction_json);
      const confidences = JSON.parse(row.confidences_json);
      setResult({
        extraction_id: row.id,
        document_id: row.document_id,
        extraction,
        confidences,
        mean_confidence: row.mean_confidence,
        latency_ms: row.latency_ms,
        model_version: row.model_version,
        schema_id: row.schema_id,
        schema_version: row.schema_version,
        doc_meta: null,
        error: null,
      });
      setSchemaId(row.schema_id);
      setCurrentFile(row.document_name);
    });
  }, [routeId]);

  const handleFile = useCallback(
    async (path: string) => {
      setError(null);
      setResult(null);
      setCurrentFile(path);
      setExtracting(true);
      try {
        const r = await ipc.extractDocument(path, schemaId);
        if (r.error) {
          setError(`${r.error.code}: ${r.error.message}`);
        } else {
          setResult(r);
        }
      } catch (e: any) {
        setError(String(e));
      } finally {
        setExtracting(false);
      }
    },
    [schemaId]
  );

  const onDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragOver(false);
      const f = e.dataTransfer.files?.[0];
      if (!f) return;
      // Tauri 2 webview drops: use the path attribute
      const path = (f as any).path as string | undefined;
      if (path) handleFile(path);
      else setError("Drag-and-drop didn't include a file path; use 'Open file' instead.");
    },
    [handleFile]
  );

  const onPickFile = useCallback(async () => {
    const path = await openDialog({
      multiple: false,
      filters: [
        { name: "Documents", extensions: ["pdf", "jpg", "jpeg", "png", "tiff", "tif", "heic", "webp", "txt"] },
      ],
    });
    if (typeof path === "string") handleFile(path);
  }, [handleFile]);

  return (
    <div>
      <header style={{ marginBottom: 24 }}>
        <h1 style={{ margin: 0, fontSize: 24 }}>Extract</h1>
        <p style={{ margin: "4px 0 0 0", color: "var(--color-fg-muted)" }}>
          Drop a document, pick a schema, and click Extract. All processing is local.
        </p>
      </header>

      <div style={{ display: "flex", gap: 16, alignItems: "center", marginBottom: 16 }}>
        <label style={{ flex: "0 0 auto" }}>
          <span style={{ marginRight: 8 }}>Schema:</span>
          <select
            value={schemaId}
            onChange={(e) => setSchemaId(e.target.value)}
            style={{ width: 240 }}
          >
            {schemas.map((s) => (
              <option key={s.id} value={s.id}>
                {s.name} ({s.id})
              </option>
            ))}
          </select>
        </label>
        <button onClick={onPickFile}>📂 Open file</button>
        {result && (
          <span style={{ color: "var(--color-fg-muted)" }}>
            mean confidence: <strong>{(result.mean_confidence * 100).toFixed(0)}%</strong> · {result.latency_ms}ms
          </span>
        )}
      </div>

      {!result && (
        <div
          onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
          onDragLeave={() => setDragOver(false)}
          onDrop={onDrop}
          onClick={onPickFile}
          style={{
            border: `2px dashed ${dragOver ? "var(--color-accent)" : "var(--color-border)"}`,
            borderRadius: "var(--radius-lg)",
            padding: 64,
            textAlign: "center",
            cursor: "pointer",
            background: dragOver ? "var(--color-bg-sunken)" : "var(--color-bg-elevated)",
            transition: "all 120ms",
          }}
        >
          {extracting ? (
            <p>Extracting…</p>
          ) : (
            <>
              <div style={{ fontSize: 48, marginBottom: 8 }}>📥</div>
              <div style={{ fontSize: 18, fontWeight: 500 }}>Drop a document here</div>
              <div style={{ color: "var(--color-fg-muted)", marginTop: 4 }}>
                or click to choose · PDF, JPEG, PNG, TIFF, HEIC, TXT
              </div>
            </>
          )}
        </div>
      )}

      {error && (
        <div
          style={{
            background: "var(--color-bg-elevated)",
            border: "1px solid var(--color-danger)",
            color: "var(--color-danger)",
            padding: 12,
            borderRadius: "var(--radius-sm)",
            marginTop: 16,
          }}
        >
          {error}
        </div>
      )}

      {result && (
        <ExtractionView
          result={result}
          schemaFields={schemas.find((s) => s.id === result.schema_id)?.fields ?? []}
          onExportExcel={async () => {
            const out = await saveDialog({
              defaultPath: `${currentFile ?? "extraction"}-${result.schema_id}.xlsx`,
              filters: [{ name: "Excel", extensions: ["xlsx"] }],
            });
            if (out) {
              await ipc.exportExcel([result.extraction_id], out);
            }
          }}
          onExportJson={async () => {
            const out = await saveDialog({
              defaultPath: `${currentFile ?? "extraction"}-${result.schema_id}.json`,
              filters: [{ name: "JSON", extensions: ["json"] }],
            });
            if (out) {
              await ipc.exportJson([result.extraction_id], out);
            }
          }}
          onExportCsv={async () => {
            const out = await saveDialog({
              defaultPath: `${currentFile ?? "extraction"}-${result.schema_id}.csv`,
              filters: [{ name: "CSV", extensions: ["csv"] }],
            });
            if (out) {
              await ipc.exportCsv([result.extraction_id], out);
            }
          }}
          onReExtract={() => handleFile(currentFile!)}
          onEdit={async (fieldPath, oldVal, newVal) => {
            await ipc.recordEdit(result.extraction_id, fieldPath, oldVal, newVal);
            setResult({
              ...result,
              extraction: { ...result.extraction!, [fieldPath]: newVal },
            });
          }}
          onReset={() => { setResult(null); setCurrentFile(null); }}
        />
      )}
    </div>
  );
}
