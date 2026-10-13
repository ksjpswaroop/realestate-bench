//! User-defined schema storage + tag operations.
//!
//! See `docs/user-schemas-and-tags.md` for the design. Built-in schemas
//! (lease addendum, invoice, rent roll, medical SOAP) are NOT stored here —
//! they remain hardcoded in the Rust SCHEMAS const and the Python
//! SCHEMA_REGISTRY. Only schemas the user creates in the desktop app land
//! in the `user_schemas` table.
//!
//! Tags are denormalized into `extractions.tags_json` (a JSON array of
//! strings). The list_all_tags / filter_by_tag paths use SQLite JSON
//! functions to query without an auxiliary table.

use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};

// ---------------------------------------------------------------------------
// Public row types (serialized to the frontend)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSchemaRow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub fields_json: String, // JSON array of UserFieldDef
    pub created_at: String,
    pub updated_at: String,
    pub is_archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserFieldType {
    ShortText,
    LongText,
    Number,
    Date,
    YesNo,
    Choice,
    List,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFieldDef {
    pub name: String,        // machine name, e.g. "vendor_name"
    pub label: String,       // display label, e.g. "Vendor Name"
    #[serde(rename = "type")]
    pub field_type: UserFieldType,
    pub description: String, // optional prompt for the LLM
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub options: Vec<String>, // for Choice
    #[serde(default)]
    pub min: Option<f64>,     // for Number
    #[serde(default)]
    pub max: Option<f64>,     // for Number
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUserSchema {
    pub name: String,
    pub description: String,
    pub fields: Vec<UserFieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateUserSchema {
    pub name: Option<String>,
    pub description: Option<String>,
    pub fields: Option<Vec<UserFieldDef>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TagCount {
    pub tag: String,
    pub count: i64,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

const MAX_FIELDS: usize = 50;
const MAX_NAME_LEN: usize = 80;
const MAX_LABEL_LEN: usize = 80;
const MAX_DESC_LEN: usize = 2000;
const MAX_FIELD_NAME_LEN: usize = 64;

pub fn validate_new(s: &NewUserSchema) -> AppResult<()> {
    let name = s.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("schema name cannot be empty".into()));
    }
    if name.len() > MAX_NAME_LEN {
        return Err(AppError::Validation(format!("schema name too long (>{MAX_NAME_LEN})")));
    }
    if s.description.len() > MAX_DESC_LEN {
        return Err(AppError::Validation(format!("description too long (>{MAX_DESC_LEN})")));
    }
    if s.fields.is_empty() {
        return Err(AppError::Validation("schema must have at least one field".into()));
    }
    if s.fields.len() > MAX_FIELDS {
        return Err(AppError::Validation(format!("too many fields (>{MAX_FIELDS})")));
    }
    let mut seen_names = std::collections::HashSet::new();
    for (i, f) in s.fields.iter().enumerate() {
        validate_field(f, i, &mut seen_names)?;
    }
    Ok(())
}

fn validate_field(
    f: &UserFieldDef,
    idx: usize,
    seen_names: &mut std::collections::HashSet<String>,
) -> AppResult<()> {
    let label = f.label.trim();
    if label.is_empty() {
        return Err(AppError::Validation(format!("field #{idx}: label is empty")));
    }
    if label.len() > MAX_LABEL_LEN {
        return Err(AppError::Validation(format!("field #{idx}: label too long")));
    }
    let name = f.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation(format!("field #{idx}: machine name is empty")));
    }
    if name.len() > MAX_FIELD_NAME_LEN {
        return Err(AppError::Validation(format!("field #{idx}: name too long")));
    }
    // machine name must be snake_case ASCII
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err(AppError::Validation(format!(
            "field #{idx}: name must be snake_case (a-z, 0-9, underscore); got {name:?}"
        )));
    }
    if !seen_names.insert(name.to_string()) {
        return Err(AppError::Validation(format!("field #{idx}: duplicate name {name:?}")));
    }
    if f.description.len() > 500 {
        return Err(AppError::Validation(format!("field #{idx}: description too long (>500)")));
    }
    if matches!(f.field_type, UserFieldType::Choice) {
        if f.options.is_empty() {
            return Err(AppError::Validation(format!(
                "field #{idx}: choice fields need at least one option"
            )));
        }
        if f.options.len() > 20 {
            return Err(AppError::Validation(format!("field #{idx}: too many options (>20)")));
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_field_name(s: &str) -> AppResult<String> {
    let name = s.trim();
    if name.is_empty() {
        return Err(AppError::Validation("field name cannot be empty".into()));
    }
    if name.len() > MAX_FIELD_NAME_LEN {
        return Err(AppError::Validation(format!("field name too long (>{MAX_FIELD_NAME_LEN})")));
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err(AppError::Validation(format!("invalid field name: {name:?}")));
    }
    Ok(name.to_string())
}

fn normalize_tag(s: &str) -> AppResult<String> {
    let t = s.trim().to_lowercase();
    if t.is_empty() {
        return Err(AppError::Validation("tag cannot be empty".into()));
    }
    if t.len() > 40 {
        return Err(AppError::Validation("tag too long (>40)".into()));
    }
    if !t.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ') {
        return Err(AppError::Validation("tag may only contain letters, numbers, spaces, '-', '_'".into()));
    }
    // Collapse internal whitespace
    Ok(t.split_whitespace().collect::<Vec<_>>().join(" "))
}

// ---------------------------------------------------------------------------
// User-schema CRUD
// ---------------------------------------------------------------------------

fn generate_id() -> String {
    // 6-char base36 suffix, prefixed with "u_" for "user"
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let n = (nanos as u64) % 0xFFFFFFFF;
    format!("u_{:08x}", n & 0xFFFFFFFF)
}

pub fn create_user_schema(s: NewUserSchema) -> AppResult<UserSchemaRow> {
    validate_new(&s)?;
    let id = generate_id();
    let fields_json = serde_json::to_string(&s.fields)?;
    let now = Utc::now().to_rfc3339();
    crate::db::with_conn(|c| {
        c.execute(
            "INSERT INTO user_schemas (id, name, description, fields_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![id, s.name.trim(), s.description, fields_json, now],
        )?;
        Ok(())
    })?;
    get_user_schema(&id)?.ok_or_else(|| AppError::Internal("just-inserted schema not found".into()))
}

pub fn get_user_schema(id: &str) -> AppResult<Option<UserSchemaRow>> {
    crate::db::with_conn(|c| {
        let row: Result<Option<UserSchemaRow>, rusqlite::Error> = c.query_row(
            "SELECT id, name, description, fields_json, created_at, updated_at, is_archived
             FROM user_schemas WHERE id = ?1",
            params![id],
            |r| Ok(map_user_schema(r)),
        )
        .optional();
        Ok(row?)
    })
}

pub fn list_user_schemas(include_archived: bool) -> AppResult<Vec<UserSchemaRow>> {
    crate::db::with_conn(|c| {
        let sql = if include_archived {
            "SELECT id, name, description, fields_json, created_at, updated_at, is_archived
             FROM user_schemas ORDER BY name COLLATE NOCASE"
        } else {
            "SELECT id, name, description, fields_json, created_at, updated_at, is_archived
             FROM user_schemas WHERE is_archived = 0 ORDER BY name COLLATE NOCASE"
        };
        let mut stmt = c.prepare(sql)?;
        let rows = stmt.query_map([], |r| Ok(map_user_schema(r)))?;
        let mut out = vec![];
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })
}

fn map_user_schema(r: &rusqlite::Row<'_>) -> UserSchemaRow {
    UserSchemaRow {
        id: r.get(0).unwrap(),
        name: r.get(1).unwrap(),
        description: r.get(2).unwrap(),
        fields_json: r.get(3).unwrap(),
        created_at: r.get(4).unwrap(),
        updated_at: r.get(5).unwrap(),
        is_archived: r.get::<_, i64>(6).unwrap() != 0,
    }
}

pub fn update_user_schema(id: &str, u: UpdateUserSchema) -> AppResult<UserSchemaRow> {
    // Pre-validate: the new state must be internally consistent.
    if let Some(name) = &u.name {
        if name.trim().is_empty() {
            return Err(AppError::Validation("name cannot be empty".into()));
        }
        if name.len() > MAX_NAME_LEN {
            return Err(AppError::Validation(format!("name too long (>{MAX_NAME_LEN})")));
        }
    }
    if let Some(fields) = &u.fields {
        if fields.is_empty() {
            return Err(AppError::Validation("fields cannot be empty".into()));
        }
        if fields.len() > MAX_FIELDS {
            return Err(AppError::Validation(format!("too many fields (>{MAX_FIELDS})")));
        }
        let mut seen = std::collections::HashSet::new();
        for (i, f) in fields.iter().enumerate() {
            validate_field(f, i, &mut seen)?;
        }
    }
    if u.name.is_none() && u.description.is_none() && u.fields.is_none() {
        return Err(AppError::Validation("no fields to update".into()));
    }

    let now = Utc::now().to_rfc3339();
    let fields_json = u.fields.as_ref().map(serde_json::to_string).transpose()?;

    crate::db::with_conn(|c| {
        // Build a dynamic UPDATE with only the provided fields
        let mut sql = String::from("UPDATE user_schemas SET updated_at = ?1");
        let mut binds: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.clone())];
        if let Some(name) = &u.name {
            sql.push_str(", name = ?");
            binds.push(Box::new(name.trim().to_string()));
        }
        if let Some(desc) = &u.description {
            sql.push_str(", description = ?");
            binds.push(Box::new(desc.clone()));
        }
        if let Some(fj) = &fields_json {
            sql.push_str(", fields_json = ?");
            binds.push(Box::new(fj.clone()));
        }
        sql.push_str(" WHERE id = ?");
        binds.push(Box::new(id.to_string()));

        let params_refs: Vec<&dyn rusqlite::ToSql> = binds.iter().map(|b| b.as_ref()).collect();
        let n = c.execute(&sql, rusqlite::params_from_iter(params_refs.iter().copied()))?;
        if n == 0 {
            return Err(AppError::NotFound(format!("user schema {id}")));
        }
        Ok(())
    })?;
    get_user_schema(id)?.ok_or_else(|| AppError::NotFound(format!("user schema {id}")))
}

pub fn delete_user_schema(id: &str) -> AppResult<()> {
    crate::db::with_conn(|c| {
        // Soft-delete: keep extractions that used this schema valid (they
        // store the denormalized schema definition in tags_json + the
        // extraction_json), so we just hide the schema from the picker.
        let n = c.execute(
            "UPDATE user_schemas SET is_archived = 1, updated_at = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), id],
        )?;
        if n == 0 {
            return Err(AppError::NotFound(format!("user schema {id}")));
        }
        Ok(())
    })
}

// ---------------------------------------------------------------------------
// Tags
// ---------------------------------------------------------------------------

pub fn get_tags(extraction_id: i64) -> AppResult<Vec<String>> {
    crate::db::with_conn(|c| {
        let raw: Option<String> = c
            .query_row(
                "SELECT tags_json FROM extractions WHERE id = ?1",
                params![extraction_id],
                |r| r.get(0),
            )
            .optional()?;
        let raw = raw.unwrap_or_else(|| "[]".to_string());
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap_or(serde_json::json!([]));
        let tags: Vec<String> = serde_json::from_value(v).unwrap_or_default();
        Ok(tags)
    })
}

pub fn add_tag(extraction_id: i64, tag: &str) -> AppResult<Vec<String>> {
    let tag = normalize_tag(tag)?;
    crate::db::with_conn(|c| {
        let raw: String = c
            .query_row(
                "SELECT tags_json FROM extractions WHERE id = ?1",
                params![extraction_id],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or_else(|| "[]".to_string());
        let mut tags: Vec<String> = serde_json::from_str(&raw).unwrap_or_default();
        if !tags.contains(&tag) {
            tags.push(tag.clone());
        }
        let new_json = serde_json::to_string(&tags)?;
        let n = c.execute(
            "UPDATE extractions SET tags_json = ?1 WHERE id = ?2",
            params![new_json, extraction_id],
        )?;
        if n == 0 {
            return Err(AppError::NotFound(format!("extraction {extraction_id}")));
        }
        Ok(tags)
    })
}

pub fn remove_tag(extraction_id: i64, tag: &str) -> AppResult<Vec<String>> {
    let tag = normalize_tag(tag)?;
    crate::db::with_conn(|c| {
        let raw: String = c
            .query_row(
                "SELECT tags_json FROM extractions WHERE id = ?1",
                params![extraction_id],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or_else(|| "[]".to_string());
        let mut tags: Vec<String> = serde_json::from_str(&raw).unwrap_or_default();
        tags.retain(|t| t != &tag);
        let new_json = serde_json::to_string(&tags)?;
        let n = c.execute(
            "UPDATE extractions SET tags_json = ?1 WHERE id = ?2",
            params![new_json, extraction_id],
        )?;
        if n == 0 {
            return Err(AppError::NotFound(format!("extraction {extraction_id}")));
        }
        Ok(tags)
    })
}

pub fn list_all_tags() -> AppResult<Vec<TagCount>> {
    crate::db::with_conn(|c| {
        // Walk every extraction's tags_json. SQLite's JSON1 makes this
        // straightforward via json_each.
        let mut stmt = c.prepare(
            "SELECT t.value AS tag, COUNT(*) AS cnt
             FROM extractions e, json_each(e.tags_json) t
             WHERE e.deleted_at IS NULL
             GROUP BY t.value
             ORDER BY cnt DESC, t.value COLLATE NOCASE ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(TagCount {
                tag: r.get(0)?,
                count: r.get(1)?,
            })
        })?;
        let mut out = vec![];
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })
}

/// Convert a user-defined field list into a Pydantic class definition (sent
/// to the sidecar) plus a JSON Schema (stored in the user schema record).
pub fn user_fields_to_json_schema(fields: &[UserFieldDef]) -> serde_json::Value {
    let mut props = serde_json::Map::new();
    let mut required = vec![];
    for f in fields {
        let mut schema = serde_json::Map::new();
        let (ty, fmt) = match f.field_type {
            UserFieldType::ShortText | UserFieldType::LongText => ("string", None),
            UserFieldType::Number => ("number", None),
            UserFieldType::Date => ("string", Some("date")),
            UserFieldType::YesNo => ("boolean", None),
            UserFieldType::List => ("array", None),
            UserFieldType::Choice => ("string", None),
        };
        schema.insert("type".into(), serde_json::Value::String(ty.to_string()));
        if let Some(f) = fmt {
            schema.insert("format".into(), serde_json::Value::String(f.to_string()));
        }
        if !f.description.is_empty() {
            schema.insert("description".into(), serde_json::Value::String(f.description.clone()));
        }
        if matches!(f.field_type, UserFieldType::Number) {
            if let Some(min) = f.min {
                schema.insert("minimum".into(), serde_json::json!(min));
            }
            if let Some(max) = f.max {
                schema.insert("maximum".into(), serde_json::json!(max));
            }
        }
        if matches!(f.field_type, UserFieldType::Choice) {
            let options: Vec<serde_json::Value> =
                f.options.iter().map(|s| serde_json::Value::String(s.clone())).collect();
            schema.insert("enum".into(), serde_json::Value::Array(options));
        }
        if matches!(f.field_type, UserFieldType::List) {
            schema.insert("items".into(), serde_json::json!({"type": "string"}));
        }
        props.insert(f.name.clone(), serde_json::Value::Object(schema));
        if f.required {
            required.push(serde_json::Value::String(f.name.clone()));
        }
    }
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": required,
        "properties": props,
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn field(name: &str, label: &str, ty: UserFieldType) -> UserFieldDef {
        UserFieldDef {
            name: name.into(),
            label: label.into(),
            field_type: ty,
            description: String::new(),
            required: false,
            options: vec![],
            min: None,
            max: None,
        }
    }

    #[test]
    fn validate_field_rejects_bad_name() {
        let mut seen = std::collections::HashSet::new();
        let bad = field("Vendor Name", "X", UserFieldType::ShortText);
        let err = validate_field(&bad, 0, &mut seen).unwrap_err();
        assert!(format!("{err}").contains("snake_case"));
    }

    #[test]
    fn validate_field_rejects_duplicate_names() {
        let mut seen = std::collections::HashSet::new();
        let a = field("foo", "A", UserFieldType::ShortText);
        validate_field(&a, 0, &mut seen).unwrap();
        let b = field("foo", "B", UserFieldType::ShortText);
        let err = validate_field(&b, 1, &mut seen).unwrap_err();
        assert!(format!("{err}").contains("duplicate"));
    }

    #[test]
    fn validate_choice_requires_options() {
        let mut seen = std::collections::HashSet::new();
        let c = field("status", "Status", UserFieldType::Choice);
        let err = validate_field(&c, 0, &mut seen).unwrap_err();
        // The validation message must mention "option" (singular) to guide the user
        assert!(format!("{err}").contains("option"), "got: {err}");
    }

    #[test]
    fn normalize_tag_lowercases_and_trims() {
        assert_eq!(normalize_tag("  Q3 2026 ").unwrap(), "q3 2026");
        assert_eq!(normalize_tag("Disputed").unwrap(), "disputed");
    }

    #[test]
    fn normalize_tag_rejects_garbage() {
        assert!(normalize_tag("hello!").is_err());
        assert!(normalize_tag("   ").is_err());
        let long = "a".repeat(41);
        assert!(normalize_tag(&long).is_err());
    }

    #[test]
    fn json_schema_includes_required() {
        let fields = vec![
            UserFieldDef {
                name: "vendor".into(),
                label: "Vendor".into(),
                field_type: UserFieldType::ShortText,
                description: "who".into(),
                required: true,
                options: vec![],
                min: None,
                max: None,
            },
            UserFieldDef {
                name: "status".into(),
                label: "Status".into(),
                field_type: UserFieldType::Choice,
                description: String::new(),
                required: false,
                options: vec!["Paid".into(), "Due".into()],
                min: None,
                max: None,
            },
        ];
        let s = user_fields_to_json_schema(&fields);
        assert_eq!(s["required"][0], "vendor");
        assert_eq!(s["properties"]["status"]["enum"][0], "Paid");
    }
}
