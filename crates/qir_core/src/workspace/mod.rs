use std::path::Path;

use rusqlite::Connection;

use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct WorkspaceMetadata {
    pub db_path: String,
    pub is_empty: bool,
}

fn validate_db_path(path: &Path) -> Result<(), AppError> {
    if path.as_os_str().is_empty() {
        return Err(AppError::new(
            "WORKSPACE_INVALID_PATH",
            "Workspace DB path is empty",
        ));
    }
    if path.exists() && path.is_dir() {
        return Err(AppError::new(
            "WORKSPACE_INVALID_PATH",
            "Workspace DB path must be a file (not a directory)",
        )
        .with_details(path.display().to_string()));
    }
    Ok(())
}

fn is_empty_conn(conn: &Connection) -> Result<bool, AppError> {
    let incidents: i64 = conn
        .query_row("SELECT COUNT(*) FROM incidents", [], |row| row.get(0))
        .map_err(|e| {
            AppError::new(
                "DB_QUERY_FAILED",
                "Failed to count incidents for workspace emptiness check",
            )
            .with_details(e.to_string())
        })?;
    let events: i64 = conn
        .query_row("SELECT COUNT(*) FROM timeline_events", [], |row| row.get(0))
        .map_err(|e| {
            AppError::new(
                "DB_QUERY_FAILED",
                "Failed to count timeline events for workspace emptiness check",
            )
            .with_details(e.to_string())
        })?;
    let artifacts: i64 = conn
        .query_row("SELECT COUNT(*) FROM artifacts", [], |row| row.get(0))
        .map_err(|e| {
            AppError::new(
                "DB_QUERY_FAILED",
                "Failed to count artifacts for workspace emptiness check",
            )
            .with_details(e.to_string())
        })?;
    Ok(incidents == 0 && events == 0 && artifacts == 0)
}

pub fn open_workspace_connection(db_path: &Path) -> Result<Connection, AppError> {
    validate_db_path(db_path)?;

    if !db_path.exists() {
        return Err(AppError::new(
            "WORKSPACE_DB_NOT_FOUND",
            "Workspace database file not found",
        )
        .with_details(db_path.display().to_string()));
    }
    if !db_path.is_file() {
        return Err(AppError::new(
            "WORKSPACE_INVALID_PATH",
            "Workspace DB path must point to a file",
        )
        .with_details(db_path.display().to_string()));
    }

    let mut conn = crate::db::open(db_path).map_err(|e| {
        // `crate::db::open` already returns AppError; remap to workspace-specific code.
        let details = e.details.clone().unwrap_or_else(|| e.to_string());
        AppError::new("WORKSPACE_OPEN_FAILED", "Failed to open workspace database")
            .with_details(details)
    })?;

    crate::db::migrate(&mut conn).map_err(|e| {
        let details = e.details.clone().unwrap_or_else(|| e.to_string());
        AppError::new(
            "WORKSPACE_MIGRATION_FAILED",
            "Failed to migrate workspace database",
        )
        .with_details(details)
    })?;

    Ok(conn)
}

pub fn create_workspace_connection(db_path: &Path) -> Result<Connection, AppError> {
    validate_db_path(db_path)?;

    if db_path.exists() {
        return Err(AppError::new(
            "WORKSPACE_CREATE_FAILED",
            "Workspace DB file already exists",
        )
        .with_details(db_path.display().to_string()));
    }

    let parent = db_path.parent().ok_or_else(|| {
        AppError::new(
            "WORKSPACE_INVALID_PATH",
            "Workspace DB path must have a parent directory",
        )
        .with_details(db_path.display().to_string())
    })?;
    std::fs::create_dir_all(parent).map_err(|e| {
        AppError::new(
            "WORKSPACE_CREATE_FAILED",
            "Failed to create workspace directory",
        )
        .with_details(format!("path={}; err={}", parent.display(), e))
    })?;

    // Opening a non-existent SQLite path creates the file.
    let mut conn = crate::db::open(db_path).map_err(|e| {
        let details = e.details.clone().unwrap_or_else(|| e.to_string());
        AppError::new("WORKSPACE_CREATE_FAILED", "Failed to create workspace database")
            .with_details(details)
    })?;

    crate::db::migrate(&mut conn).map_err(|e| {
        let details = e.details.clone().unwrap_or_else(|| e.to_string());
        AppError::new(
            "WORKSPACE_MIGRATION_FAILED",
            "Failed to migrate newly created workspace database",
        )
        .with_details(details)
    })?;

    Ok(conn)
}

pub fn open_workspace(db_path: &Path) -> Result<WorkspaceMetadata, AppError> {
    let conn = open_workspace_connection(db_path)?;
    let empty = is_empty_conn(&conn)?;
    Ok(WorkspaceMetadata {
        db_path: db_path.to_string_lossy().to_string(),
        is_empty: empty,
    })
}

pub fn create_workspace(db_path: &Path) -> Result<WorkspaceMetadata, AppError> {
    let conn = create_workspace_connection(db_path)?;
    let empty = is_empty_conn(&conn)?;
    Ok(WorkspaceMetadata {
        db_path: db_path.to_string_lossy().to_string(),
        is_empty: empty,
    })
}

pub fn db_is_empty(db_path: &Path) -> Result<bool, AppError> {
    let conn = open_workspace_connection(db_path)?;
    is_empty_conn(&conn)
}
