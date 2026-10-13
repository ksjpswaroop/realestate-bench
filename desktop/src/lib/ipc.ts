/** Tauri IPC helper. All commands defined in src-tauri/src/commands.rs */
import { invoke } from "@tauri-apps/api/core";

export type SchemaInfo = {
  id: string;
  name: string;
  doc: string;
  fields: string[];
  version: string;
};

export type ExtractResult = {
  extraction_id: number;
  document_id: number;
  extraction: Record<string, unknown> | null;
  confidences: Record<string, number>;
  mean_confidence: number;
  latency_ms: number;
  model_version: string;
  schema_id: string;
  schema_version: string;
  doc_meta: Record<string, unknown> | null;
  error: { code: string; message: string } | null;
};

export type ExtractionRow = {
  id: number;
  document_id: number;
  schema_id: string;
  schema_version: string;
  extraction_json: string;
  confidences_json: string;
  mean_confidence: number;
  latency_ms: number;
  model_version: string;
  created_at: string;
  deleted_at: string | null;
  document_path: string;
  document_name: string;
};

export type DbInfo = {
  path: string;
  document_count: number;
  extraction_count: number;
  edit_count: number;
  size_bytes: number;
};

export const ipc = {
  listSchemas: () => invoke<SchemaInfo[]>("list_schemas"),
  extractDocument: (doc_path: string, schema_id: string) =>
    invoke<ExtractResult>("extract_document", { args: { doc_path, schema_id } }),
  listExtractions: (schema_id?: string, search?: string) =>
    invoke<ExtractionRow[]>("list_extractions", { args: { schema_id, search, limit: 200 } }),
  getExtraction: (id: number) => invoke<ExtractionRow>("get_extraction", { id }),
  deleteExtraction: (id: number) => invoke<void>("delete_extraction", { id }),
  recordEdit: (extraction_id: number, field_path: string, old_value: unknown, new_value: unknown) =>
    invoke<number>("record_edit", {
      args: { extraction_id, field_path, old_value, new_value },
    }),
  searchExtractions: (query: string) =>
    invoke<ExtractionRow[]>("search_extractions", { query }),
  exportExcel: (extraction_ids: number[], out_path: string) =>
    invoke<string>("export_excel", { extractionIds: extraction_ids, outPath: out_path }),
  exportJson: (extraction_ids: number[], out_path: string) =>
    invoke<string>("export_json", { extractionIds: extraction_ids, outPath: out_path }),
  exportCsv: (extraction_ids: number[], out_path: string) =>
    invoke<string>("export_csv", { extractionIds: extraction_ids, outPath: out_path }),
  dbInfo: () => invoke<DbInfo>("db_info"),
  wipeDb: () => invoke<void>("wipe_db"),
};
