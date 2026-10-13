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

// Phase 2: User-defined schemas + tags
export type UserFieldType =
  | "short_text"
  | "long_text"
  | "number"
  | "date"
  | "yes_no"
  | "choice"
  | "list";

export type UserFieldDef = {
  name: string;
  label: string;
  type: UserFieldType;
  description: string;
  required: boolean;
  options: string[];
  min: number | null;
  max: number | null;
};

export type UserSchemaRow = {
  id: string;
  name: string;
  description: string;
  fields_json: string;
  created_at: string;
  updated_at: string;
  is_archived: boolean;
};

export type TagCount = {
  tag: string;
  count: number;
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

  // --- Phase 2: user schemas ---
  listUserSchemas: (include_archived = false) =>
    invoke<UserSchemaRow[]>("list_user_schemas", { includeArchived: include_archived }),
  getUserSchema: (id: string) =>
    invoke<UserSchemaRow | null>("get_user_schema", { id }),
  createUserSchema: (name: string, description: string, fields: UserFieldDef[]) =>
    invoke<UserSchemaRow>("create_user_schema", { name, description, fields }),
  updateUserSchema: (
    id: string,
    name?: string,
    description?: string,
    fields?: UserFieldDef[],
  ) =>
    invoke<UserSchemaRow>("update_user_schema", {
      id,
      name: name ?? null,
      description: description ?? null,
      fields: fields ?? null,
    }),
  deleteUserSchema: (id: string) => invoke<void>("delete_user_schema", { id }),

  // --- Phase 2: tags ---
  getExtractionTags: (extraction_id: number) =>
    invoke<string[]>("get_extraction_tags", { extractionId: extraction_id }),
  addExtractionTag: (extraction_id: number, tag: string) =>
    invoke<string[]>("add_extraction_tag", { extractionId: extraction_id, tag }),
  removeExtractionTag: (extraction_id: number, tag: string) =>
    invoke<string[]>("remove_extraction_tag", { extractionId: extraction_id, tag }),
  listAllTags: () => invoke<TagCount[]>("list_all_tags"),
  filterExtractionsByTag: (tag: string) =>
    invoke<number[]>("filter_extractions_by_tag", { tag }),
};
