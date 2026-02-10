use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::ingest::jira_csv::JiraCsvMapping;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JiraMappingProfile {
    pub id: i64,
    pub name: String,
    pub mapping: JiraCsvMapping,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JiraMappingProfileUpsert {
    pub id: Option<i64>,
    pub name: String,
    pub mapping: JiraCsvMapping,
}

pub fn list_profiles(conn: &Connection) -> Result<Vec<JiraMappingProfile>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, mapping_json FROM jira_mapping_profiles ORDER BY name ASC, id ASC",
        )
        .map_err(|e| {
            AppError::new(
                "DB_QUERY_FAILED",
                "Failed to prepare Jira mapping profile list query",
            )
            .with_details(e.to_string())
        })?;

    let rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let mapping_json: String = row.get(2)?;
            Ok((id, name, mapping_json))
        })
        .map_err(|e| {
            AppError::new("DB_QUERY_FAILED", "Failed to query Jira mapping profiles")
                .with_details(e.to_string())
        })?;

    let mut out = Vec::new();
    for r in rows {
        let (id, name, mapping_json) = r.map_err(|e| {
            AppError::new(
                "DB_QUERY_FAILED",
                "Failed to decode Jira mapping profile row",
            )
            .with_details(e.to_string())
        })?;
        let mapping: JiraCsvMapping = serde_json::from_str(&mapping_json).map_err(|e| {
            AppError::new(
                "DB_DECODE_FAILED",
                "Failed to decode Jira mapping profile JSON",
            )
            .with_details(e.to_string())
        })?;
        out.push(JiraMappingProfile { id, name, mapping });
    }
    Ok(out)
}

pub fn get_profile(conn: &Connection, id: i64) -> Result<JiraMappingProfile, AppError> {
    let (name, mapping_json): (String, String) = conn
        .query_row(
            "SELECT name, mapping_json FROM jira_mapping_profiles WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| {
            AppError::new("DB_NOT_FOUND", "Jira mapping profile not found")
                .with_details(e.to_string())
        })?;

    let mapping: JiraCsvMapping = serde_json::from_str(&mapping_json).map_err(|e| {
        AppError::new(
            "DB_DECODE_FAILED",
            "Failed to decode Jira mapping profile JSON",
        )
        .with_details(e.to_string())
    })?;

    Ok(JiraMappingProfile { id, name, mapping })
}

pub fn upsert_profile(
    conn: &mut Connection,
    upsert: JiraMappingProfileUpsert,
) -> Result<JiraMappingProfile, AppError> {
    if upsert.name.trim().is_empty() {
        return Err(AppError::new(
            "VALIDATION_MAPPING_PROFILE_INVALID",
            "Profile name is required",
        ));
    }
    if upsert.mapping.title.trim().is_empty() {
        return Err(AppError::new(
            "VALIDATION_MAPPING_PROFILE_INVALID",
            "Mapping must include a title column",
        ));
    }

    let mapping_json = serde_json::to_string(&upsert.mapping).map_err(|e| {
        AppError::new(
            "DB_ENCODE_FAILED",
            "Failed to encode Jira mapping profile JSON",
        )
        .with_details(e.to_string())
    })?;

    if let Some(id) = upsert.id {
        let changed = conn
            .execute(
                "UPDATE jira_mapping_profiles SET name = ?1, mapping_json = ?2 WHERE id = ?3",
                rusqlite::params![upsert.name, mapping_json, id],
            )
            .map_err(|e| {
                AppError::new("DB_WRITE_FAILED", "Failed to update Jira mapping profile")
                    .with_details(e.to_string())
            })?;
        if changed == 0 {
            return Err(AppError::new(
                "DB_NOT_FOUND",
                "Jira mapping profile not found",
            ));
        }
        return get_profile(conn, id);
    }

    conn
    .execute(
      "INSERT INTO jira_mapping_profiles(name, mapping_json, created_at) VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
      rusqlite::params![upsert.name, mapping_json],
    )
    .map_err(|e| {
      AppError::new("DB_WRITE_FAILED", "Failed to create Jira mapping profile").with_details(e.to_string())
    })?;

    let id = conn.last_insert_rowid();
    get_profile(conn, id)
}

pub fn delete_profile(conn: &mut Connection, id: i64) -> Result<(), AppError> {
    let changed = conn
        .execute("DELETE FROM jira_mapping_profiles WHERE id = ?1", [id])
        .map_err(|e| {
            AppError::new("DB_WRITE_FAILED", "Failed to delete Jira mapping profile")
                .with_details(e.to_string())
        })?;

    if changed == 0 {
        return Err(AppError::new(
            "DB_NOT_FOUND",
            "Jira mapping profile not found",
        ));
    }
    Ok(())
}
