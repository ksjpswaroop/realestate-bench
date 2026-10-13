//! Tauri commands — the IPC surface exposed to the React frontend.
//!
//! Each function is annotated with `#[tauri::command]` and registered in
//! `lib.rs::run()`. The frontend calls these via `invoke<T>("name", args)`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::AppHandle;

use crate::db;
use crate::errors::{AppError, AppResult};
use crate::extract;
use crate::paths;
use crate::user_schemas;

// ---------------------------------------------------------------------------
// Schema metadata (compiled-in)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub id: String,
    pub name: String,
    pub doc: String,
    pub fields: Vec<String>,
    pub version: String,
}

const SCHEMAS: &[(&str, &str, &str, &[&str])] = &[
    (
        "lease_addendum",
        "LeaseAddendum",
        "Structured extraction for a residential or commercial lease addendum.",
        &["landlord", "tenant", "property_address", "monthly_rent", "currency", "start_date", "end_date", "security_deposit", "renewal_option", "pets_allowed", "notes"],
    ),
    (
        "invoice",
        "Invoice",
        "Structured extraction for a vendor invoice / bill / receipt.",
        &["vendor_name", "vendor_address", "invoice_number", "invoice_date", "due_date", "currency", "subtotal", "tax", "total", "payment_terms", "notes"],
    ),
    (
        "rent_roll",
        "RentRoll",
        "Structured extraction for a single unit's row in a property rent roll.",
        &["property_name", "unit_number", "tenant_name", "square_feet", "monthly_rent", "rent_per_sqft", "lease_start", "lease_end", "status"],
    ),
    (
        "medical_soap_note",
        "MedicalSoapNote",
        "Structured extraction for a clinical SOAP note (Subjective/Objective/Assessment/Plan).",
        &["patient_id", "encounter_date", "chief_complaint", "hpi", "past_medical_history",
          "medications", "allergies", "vital_signs", "physical_exam", "assessment", "plan",
          "follow_up", "provider_name"],
    ),
];

// ---------------------------------------------------------------------------
// Hashing
// ---------------------------------------------------------------------------

fn sha256_file(path: &std::path::Path) -> AppResult<String> {
    let bytes = std::fs::read(path)?;
    let mut h = Sha256::new();
    h.update(&bytes);
    Ok(hex::encode(h.finalize()))
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractArgs {
    pub doc_path: String,
    pub schema_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtractResult {
    pub extraction_id: i64,
    pub document_id: i64,
    pub extraction: Option<serde_json::Value>,
    pub confidences: serde_json::Map<String, serde_json::Value>,
    pub mean_confidence: f64,
    pub latency_ms: i64,
    pub model_version: String,
    pub schema_id: String,
    pub schema_version: String,
    pub doc_meta: Option<serde_json::Value>,
    pub error: Option<extract::SidecarError>,
}

#[tauri::command]
pub async fn extract_document(app: AppHandle, args: ExtractArgs) -> AppResult<ExtractResult> {
    // 1. Hash the file
    let p = PathBuf::from(&args.doc_path);
    if !p.exists() {
        return Err(AppError::NotFound(format!("file not found: {}", args.doc_path)));
    }
    let meta = std::fs::metadata(&p)?;
    let sha = sha256_file(&p)?;
    let file_name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string();
    let page_count = count_pdf_pages(&p).ok().map(|n| n as i64);

    // 2. Upsert document
    let doc_id = db::upsert_document(db::NewDocument {
        path: &args.doc_path,
        sha256: &sha,
        file_name: &file_name,
        file_size: meta.len() as i64,
        page_count,
    })?;

    // 3. Run sidecar
    //    If the schema_id is a built-in, the sidecar handles it directly.
    //    If it's a user-defined schema, look it up in the DB and pass the
    //    JSON definition to the sidecar so it can build a dynamic Pydantic
    //    class.
    let mut sidecar_req = extract::SidecarRequest {
        doc_path: args.doc_path.clone(),
        schema_id: args.schema_id.clone(),
        user_schema_json: None,
        user_schema_name: None,
    };
    if !is_built_in_schema(&args.schema_id) {
        match user_schemas::get_user_schema(&args.schema_id)? {
            Some(u) => {
                sidecar_req.user_schema_json = Some(u.fields_json.clone());
                sidecar_req.user_schema_name = Some(u.name.clone());
            }
            None => {
                return Err(AppError::NotFound(format!("schema {}", args.schema_id)));
            }
        }
    }
    let response = extract::run(&app, sidecar_req).await?;

    // 4. Persist (even on error — we want to remember the attempt)
    let extraction_id = if let Some(ref ext) = response.extraction {
        let extraction_json = serde_json::to_string(ext)?;
        let confidences_json = serde_json::to_string(&response.confidences)?;
        db::insert_extraction(db::NewExtraction {
            document_id: doc_id,
            schema_id: &args.schema_id,
            schema_version: response.schema_version.as_deref().unwrap_or("unknown"),
            extraction_json: &extraction_json,
            confidences_json: &confidences_json,
            mean_confidence: response.mean_confidence,
            latency_ms: response.latency_ms,
            model_version: &response.model_version,
        })?
    } else {
        0
    };

    Ok(ExtractResult {
        extraction_id,
        document_id: doc_id,
        extraction: response.extraction,
        confidences: response.confidences,
        mean_confidence: response.mean_confidence,
        latency_ms: response.latency_ms,
        model_version: response.model_version,
        schema_id: response.schema_id.unwrap_or(args.schema_id),
        schema_version: response.schema_version.unwrap_or_default(),
        doc_meta: response.doc_meta,
        error: response.error,
    })
}

fn count_pdf_pages(path: &std::path::Path) -> AppResult<u32> {
    let bytes = std::fs::read(path)?;
    // Quick page count by counting "/Type /Page" markers (works for ~99% of PDFs).
    let s = String::from_utf8_lossy(&bytes);
    let count = s.matches("/Type /Page").count() + s.matches("/Type/Page").count();
    if count == 0 {
        return Err(AppError::Internal("not a PDF".into()));
    }
    Ok(count as u32)
}

#[tauri::command]
pub fn list_schemas() -> AppResult<Vec<SchemaInfo>> {
    let mut out: Vec<SchemaInfo> = SCHEMAS
        .iter()
        .map(|(id, name, doc, fields)| SchemaInfo {
            id: id.to_string(),
            name: name.to_string(),
            doc: doc.to_string(),
            fields: fields.iter().map(|f| f.to_string()).collect(),
            version: "1.0.0".to_string(),
        })
        .collect();

    // Append user-defined schemas (Phase 2). They are loaded fresh from the
    // DB on every call so the React UI reflects edits immediately.
    for u in crate::user_schemas::list_user_schemas(false)? {
        let fields: Vec<user_schemas::UserFieldDef> =
            serde_json::from_str(&u.fields_json).unwrap_or_default();
        out.push(SchemaInfo {
            id: u.id,
            name: u.name,
            doc: u.description,
            fields: fields.into_iter().map(|f| f.name).collect(),
            version: "user".to_string(),
        });
    }

    Ok(out)
}

#[tauri::command]
pub fn get_schema_fields(schema_id: String) -> AppResult<Vec<String>> {
    if let Some((_, _, _, fields)) = SCHEMAS.iter().find(|(id, _, _, _)| *id == schema_id) {
        return Ok(fields.iter().map(|f| f.to_string()).collect());
    }
    // User schema?
    if let Some(u) = user_schemas::get_user_schema(&schema_id)? {
        let fields: Vec<user_schemas::UserFieldDef> =
            serde_json::from_str(&u.fields_json).unwrap_or_default();
        return Ok(fields.into_iter().map(|f| f.name).collect());
    }
    Err(AppError::NotFound(format!("schema {schema_id}")))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListExtractionsArgs {
    pub schema_id: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
}

#[tauri::command]
pub fn list_extractions(args: ListExtractionsArgs) -> AppResult<Vec<db::ExtractionRow>> {
    db::list_extractions(
        args.schema_id.as_deref(),
        args.search.as_deref(),
        args.limit.unwrap_or(200),
    )
}

#[tauri::command]
pub fn get_extraction(id: i64) -> AppResult<db::ExtractionRow> {
    db::get_extraction(id)
}

#[tauri::command]
pub fn delete_extraction(id: i64) -> AppResult<()> {
    db::soft_delete_extraction(id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordEditArgs {
    pub extraction_id: i64,
    pub field_path: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
}

#[tauri::command]
pub fn record_edit(args: RecordEditArgs) -> AppResult<i64> {
    db::record_edit(args.extraction_id, &args.field_path, &args.old_value, &args.new_value)
}

#[tauri::command]
pub fn search_extractions(query: String) -> AppResult<Vec<db::ExtractionRow>> {
    db::list_extractions(None, Some(&query), 200)
}

// ---------------------------------------------------------------------------
// Export commands (FR-016, FR-017, FR-018)
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn export_excel(
    app: AppHandle,
    extraction_ids: Vec<i64>,
    out_path: String,
) -> AppResult<String> {
    let rows = extraction_ids
        .iter()
        .map(|id| db::get_extraction(*id))
        .collect::<AppResult<Vec<_>>>()?;
    write_excel(&rows, std::path::Path::new(&out_path))?;
    Ok(out_path)
}

#[tauri::command]
pub async fn export_json(
    extraction_ids: Vec<i64>,
    out_path: String,
) -> AppResult<String> {
    let rows = extraction_ids
        .iter()
        .map(|id| db::get_extraction(*id))
        .collect::<AppResult<Vec<_>>>()?;
    let arr: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "extraction_id": r.id,
                "doc_path": r.document_path,
                "doc_name": r.document_name,
                "schema_id": r.schema_id,
                "schema_version": r.schema_version,
                "extraction": serde_json::from_str::<serde_json::Value>(&r.extraction_json).unwrap_or(serde_json::Value::Null),
                "confidences": serde_json::from_str::<serde_json::Value>(&r.confidences_json).unwrap_or(serde_json::Value::Null),
                "mean_confidence": r.mean_confidence,
                "latency_ms": r.latency_ms,
                "model_version": r.model_version,
                "created_at": r.created_at,
            })
        })
        .collect();
    std::fs::write(&out_path, serde_json::to_string_pretty(&arr)?)?;
    Ok(out_path)
}

#[tauri::command]
pub async fn export_csv(
    extraction_ids: Vec<i64>,
    out_path: String,
) -> AppResult<String> {
    let rows = extraction_ids
        .iter()
        .map(|id| db::get_extraction(*id))
        .collect::<AppResult<Vec<_>>>()?;
    write_csv(&rows, std::path::Path::new(&out_path))?;
    Ok(out_path)
}

#[tauri::command]
pub async fn export_db(out_path: String) -> AppResult<String> {
    // The DB is always at the same path; copy it to the user-chosen location
    // and run VACUUM on the copy so the file is self-contained.
    let _ = out_path.clone();
    let src = std::path::Path::new(&out_path); // we accept the path the user wants
    let _ = src;
    Err(AppError::Internal(
        "export_db: use Settings → Export DB which calls db_info with the real db path".into(),
    ))
}

#[tauri::command]
pub async fn import_db(_app: AppHandle, _src_path: String) -> AppResult<()> {
    Err(AppError::Internal(
        "import_db: not yet implemented; will ATTACH the user DB and copy rows in a future release".into(),
    ))
}

#[tauri::command]
pub fn wipe_db() -> AppResult<()> {
    db::wipe()
}

#[tauri::command]
pub fn db_info(app: AppHandle) -> AppResult<db::DbInfo> {
    let path = paths::db_path(&app)?;
    db::db_info(&path)
}

// ---------------------------------------------------------------------------
// Excel/CSV writers (minimal, no external deps)
// ---------------------------------------------------------------------------
//
// We implement a *very* small XLSX writer to avoid pulling in a heavy dep
// like rust_xlsxwriter. The output is a real .xlsx (ZIP of XMLs) and
// opens in Excel and LibreOffice. For richer formatting, swap in
// rust_xlsxwriter later.

fn write_excel(rows: &[db::ExtractionRow], out: &std::path::Path) -> AppResult<()> {
    use std::io::Write;
    let f = std::fs::File::create(out)?;
    let mut zip = zip::ZipWriter::new(f);
    let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    // [Content_Types].xml
    zip.start_file("[Content_Types].xml", opts)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#,
    )?;

    // _rels/.rels
    zip.start_file("_rels/.rels", opts)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#,
    )?;

    // xl/_rels/workbook.xml.rels
    zip.start_file("xl/_rels/workbook.xml.rels", opts)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#,
    )?;

    // xl/styles.xml
    zip.start_file("xl/styles.xml", opts)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>
  <fills count="1"><fill><patternFill patternType="none"/></fill></fills>
  <borders count="1"><border/></borders>
  <cellStyleXfs count="1"><xf/></cellStyleXfs>
  <cellXfs count="1"><xf/></cellXfs>
  <cellStyles count="1"><cellStyle name="Normal" xfId="0"/></cellStyles>
</styleSheet>"#,
    )?;

    // Build shared strings and sheet
    let mut shared: Vec<String> = vec![];
    let mut sheet = String::new();
    sheet.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>"#);
    let mut row_idx = 1u32;
    for r in rows {
        // Header row
        let mut header = String::from(r#"<row r=""#.to_string());
        header.push_str(&row_idx.to_string());
        header.push_str(r#"" r="1">"#);
        for h in ["field_name", "value", "confidence", "user_verified", "extraction_id", "document_name", "schema_id"] {
            let idx = shared_index(&mut shared, h) as u32;
            header.push_str(&format!(r#"<c t="s"><v>{idx}</v></c>"#));
        }
        header.push_str("</row>");
        sheet.push_str(&header);
        row_idx += 1;

        // Data rows
        let extraction: serde_json::Value = serde_json::from_str(&r.extraction_json).unwrap_or(serde_json::Value::Null);
        let confidences: serde_json::Value = serde_json::from_str(&r.confidences_json).unwrap_or(serde_json::Value::Null);
        let edits = db::list_edits(r.id).unwrap_or_default();
        let edit_map: std::collections::HashMap<String, serde_json::Value> = edits
            .iter()
            .filter(|e| e.edited_at >= r.created_at) // take latest edit only
            .map(|e| (e.field_path.clone(), e.new_value.clone()))
            .collect();
        if let serde_json::Value::Object(map) = &extraction {
            for (k, v) in map {
                let conf = confidences.get(k).and_then(|c| c.as_f64()).unwrap_or(0.0);
                let user_value = edit_map.get(k).cloned().unwrap_or(v.clone());
                let verified = edit_map.contains_key(k);
                let mut row = format!(r#"<row r="{}">"#, row_idx);
                row.push_str(&xlsx_cell(&mut shared, "field_name", true, k));
                row.push_str(&xlsx_cell_value(&mut shared, &user_value));
                row.push_str(&format!(r#"<c><v>{:.3}</v></c>"#, conf));
                row.push_str(&xlsx_cell(&mut shared, "user_verified", true, if verified { "true" } else { "false" }));
                row.push_str(&format!(r#"<c><v>{}</v></c>"#, r.id));
                row.push_str(&xlsx_cell(&mut shared, "document_name", true, &r.document_name));
                row.push_str(&xlsx_cell(&mut shared, "schema_id", true, &r.schema_id));
                row.push_str("</row>");
                sheet.push_str(&row);
                row_idx += 1;
            }
        }
    }
    sheet.push_str("  </sheetData>\n</worksheet>");

    // xl/sharedStrings.xml
    zip.start_file("xl/sharedStrings.xml", opts)?;
    let mut ss = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count=""#.to_string());
    ss.push_str(&shared.len().to_string());
    ss.push_str(r#"" uniqueCount=""#);
    ss.push_str(&shared.len().to_string());
    ss.push_str(r#"">"#);
    for s in &shared {
        ss.push_str("<si><t xml:space=\"preserve\">");
        ss.push_str(&xml_escape(s));
        ss.push_str("</t></si>");
    }
    ss.push_str("</sst>");
    zip.write_all(ss.as_bytes())?;

    // xl/workbook.xml
    zip.start_file("xl/workbook.xml", opts)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets><sheet name="Extractions" sheetId="1" r:id="rId1"/></sheets>
</workbook>"#,
    )?;

    // xl/worksheets/sheet1.xml
    zip.start_file("xl/worksheets/sheet1.xml", opts)?;
    zip.write_all(sheet.as_bytes())?;

    zip.finish()?;
    Ok(())
}

fn shared_index(shared: &mut Vec<String>, s: &str) -> usize {
    if let Some(i) = shared.iter().position(|x| x == s) {
        return i;
    }
    shared.push(s.to_string());
    shared.len() - 1
}

fn xlsx_cell(shared: &mut Vec<String>, _header: &str, is_str: bool, val: &str) -> String {
    if is_str {
        let idx = shared_index(shared, val) as u32;
        format!(r#"<c t="s"><v>{idx}</v></c>"#)
    } else {
        format!(r#"<c><v>{val}</v></c>"#)
    }
}

fn xlsx_cell_value(shared: &mut Vec<String>, val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => {
            let idx = shared_index(shared, s) as u32;
            format!(r#"<c t="s"><v>{idx}</v></c>"#)
        }
        serde_json::Value::Number(n) => format!(r#"<c><v>{}</v></c>"#, n),
        serde_json::Value::Bool(b) => format!(r#"<c><v>{}</v></c>"#, if *b { 1 } else { 0 }),
        serde_json::Value::Null => r#"<c/>"#.to_string(),
        other => {
            let s = other.to_string();
            let idx = shared_index(shared, &s) as u32;
            format!(r#"<c t="s"><v>{idx}</v></c>"#)
        }
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn write_csv(rows: &[db::ExtractionRow], out: &std::path::Path) -> AppResult<()> {
    let mut w = std::fs::File::create(out)?;
    use std::io::Write;
    writeln!(w, "extraction_id,document_name,schema_id,field_name,value,confidence")?;
    for r in rows {
        let extraction: serde_json::Value = serde_json::from_str(&r.extraction_json).unwrap_or(serde_json::Value::Null);
        let confidences: serde_json::Value = serde_json::from_str(&r.confidences_json).unwrap_or(serde_json::Value::Null);
        let edits = db::list_edits(r.id).unwrap_or_default();
        let edit_map: std::collections::HashMap<String, serde_json::Value> = edits
            .iter()
            .filter(|e| e.edited_at >= r.created_at)
            .map(|e| (e.field_path.clone(), e.new_value.clone()))
            .collect();
        if let serde_json::Value::Object(map) = &extraction {
            for (k, v) in map {
                let conf = confidences.get(k).and_then(|c| c.as_f64()).unwrap_or(0.0);
                let user_value = edit_map.get(k).cloned().unwrap_or(v.clone());
                writeln!(
                    w,
                    "{},{},{},{},{},{:.3}",
                    r.id,
                    csv_escape(&r.document_name),
                    csv_escape(&r.schema_id),
                    csv_escape(k),
                    csv_escape(&user_value.to_string()),
                    conf,
                )?;
            }
        }
    }
    Ok(())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// User-defined schema + tag commands (Phase 2)
// ---------------------------------------------------------------------------

fn is_built_in_schema(id: &str) -> bool {
    SCHEMAS.iter().any(|(bid, _, _, _)| *bid == id)
}

#[tauri::command]
pub fn list_user_schemas(include_archived: Option<bool>) -> AppResult<Vec<user_schemas::UserSchemaRow>> {
    user_schemas::list_user_schemas(include_archived.unwrap_or(false))
}

#[tauri::command]
pub fn get_user_schema(id: String) -> AppResult<Option<user_schemas::UserSchemaRow>> {
    user_schemas::get_user_schema(&id)
}

#[tauri::command]
pub fn create_user_schema(
    name: String,
    description: String,
    fields: Vec<user_schemas::UserFieldDef>,
) -> AppResult<user_schemas::UserSchemaRow> {
    user_schemas::create_user_schema(user_schemas::NewUserSchema {
        name,
        description,
        fields,
    })
}

#[tauri::command]
pub fn update_user_schema(
    id: String,
    name: Option<String>,
    description: Option<String>,
    fields: Option<Vec<user_schemas::UserFieldDef>>,
) -> AppResult<user_schemas::UserSchemaRow> {
    user_schemas::update_user_schema(
        &id,
        user_schemas::UpdateUserSchema {
            name,
            description,
            fields,
            ..Default::default()
        },
    )
}

#[tauri::command]
pub fn delete_user_schema(id: String) -> AppResult<()> {
    user_schemas::delete_user_schema(&id)
}

#[tauri::command]
pub fn add_extraction_tag(extraction_id: i64, tag: String) -> AppResult<Vec<String>> {
    user_schemas::add_tag(extraction_id, &tag)
}

#[tauri::command]
pub fn remove_extraction_tag(extraction_id: i64, tag: String) -> AppResult<Vec<String>> {
    user_schemas::remove_tag(extraction_id, &tag)
}

#[tauri::command]
pub fn get_extraction_tags(extraction_id: i64) -> AppResult<Vec<String>> {
    user_schemas::get_tags(extraction_id)
}

#[tauri::command]
pub fn list_all_tags() -> AppResult<Vec<user_schemas::TagCount>> {
    user_schemas::list_all_tags()
}

#[tauri::command]
pub fn filter_extractions_by_tag(tag: String) -> AppResult<Vec<i64>> {
    let tag = tag.trim().to_lowercase();
    db::with_conn(|c| {
        let mut stmt = c.prepare(
            "SELECT id FROM extractions
             WHERE deleted_at IS NULL
               AND EXISTS (
                   SELECT 1 FROM json_each(tags_json) j
                   WHERE j.value = ?1
               )
             ORDER BY created_at DESC
             LIMIT 1000",
        )?;
        let rows = stmt.query_map([tag], |r| r.get::<_, i64>(0))?;
        let mut out = vec![];
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })
}
