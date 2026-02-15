use std::collections::HashSet;
use std::path::Path;

use rusqlite::Connection;
use rusqlite::OptionalExtension;

use crate::error::AppError;

const MIGRATION_0001: (&str, &str) = (
    "0001_init.sql",
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../migrations/0001_init.sql"
    )),
);

const MIGRATION_0002: (&str, &str) = (
    "0002_add_timestamp_raw_columns.sql",
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../migrations/0002_add_timestamp_raw_columns.sql"
    )),
);

const MIGRATION_0003: (&str, &str) = (
    "0003_add_story_fields.sql",
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../migrations/0003_add_story_fields.sql"
    )),
);

const MIGRATION_0004: (&str, &str) = (
    "0004_add_ai_drafts.sql",
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../migrations/0004_add_ai_drafts.sql"
    )),
);

const MIGRATION_0005: (&str, &str) = (
    "0005_add_draft_revisions.sql",
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../migrations/0005_add_draft_revisions.sql"
    )),
);

const MIGRATION_0006: (&str, &str) = (
    "0006_add_pagination_indexes.sql",
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../migrations/0006_add_pagination_indexes.sql"
    )),
);

fn migrations() -> Vec<(&'static str, &'static str)> {
    vec![
        MIGRATION_0001,
        MIGRATION_0002,
        MIGRATION_0003,
        MIGRATION_0004,
        MIGRATION_0005,
        MIGRATION_0006,
    ]
}

pub fn latest_migration_name() -> &'static str {
    migrations()
        .last()
        .map(|(name, _)| *name)
        .unwrap_or("NONE")
}

pub fn applied_migration_names(conn: &Connection) -> Result<Vec<String>, AppError> {
    // Preflight-safe: do not create tables here.
    let exists: Option<String> = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='_migrations'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| {
            AppError::new(
                "DB_MIGRATIONS_QUERY_FAILED",
                "Failed to detect migrations table",
            )
            .with_details(e.to_string())
        })?;

    if exists.is_none() {
        return Ok(Vec::new());
    }

    let mut stmt = conn
        .prepare("SELECT name FROM _migrations ORDER BY name ASC")
        .map_err(|e| {
            AppError::new(
                "DB_MIGRATIONS_QUERY_FAILED",
                "Failed to query applied migrations",
            )
            .with_details(e.to_string())
        })?;

    let rows = stmt.query_map([], |row| row.get::<_, String>(0)).map_err(|e| {
        AppError::new(
            "DB_MIGRATIONS_QUERY_FAILED",
            "Failed to read applied migrations",
        )
        .with_details(e.to_string())
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| {
            AppError::new(
                "DB_MIGRATIONS_QUERY_FAILED",
                "Failed to read applied migration row",
            )
            .with_details(e.to_string())
        })?);
    }
    Ok(out)
}

pub fn pending_migration_names(conn: &Connection) -> Result<Vec<String>, AppError> {
    let applied = applied_migration_names(conn)?;
    let applied_set: HashSet<String> = applied.into_iter().collect();
    let mut out: Vec<String> = Vec::new();
    for (name, _) in migrations() {
        if !applied_set.contains(name) {
            out.push(name.to_string());
        }
    }
    Ok(out)
}

pub fn open(path: &Path) -> Result<Connection, AppError> {
    Connection::open(path).map_err(|e| {
        AppError::new("DB_OPEN_FAILED", "Failed to open SQLite database")
            .with_details(e.to_string())
    })
}

pub fn open_in_memory() -> Result<Connection, AppError> {
    Connection::open_in_memory().map_err(|e| {
        AppError::new("DB_OPEN_FAILED", "Failed to open in-memory SQLite database")
            .with_details(e.to_string())
    })
}

pub fn migrate(conn: &mut Connection) -> Result<(), AppError> {
    // Track migrations by name, applying each exactly once, in deterministic order.
    conn.execute_batch(
        r#"
      PRAGMA foreign_keys = ON;
      CREATE TABLE IF NOT EXISTS _migrations (
        name TEXT PRIMARY KEY NOT NULL,
        applied_at TEXT NOT NULL
      );
    "#,
    )
    .map_err(|e| {
        AppError::new(
            "DB_MIGRATIONS_TABLE_FAILED",
            "Failed to ensure migrations table exists",
        )
        .with_details(e.to_string())
    })?;

    let applied: HashSet<String> = {
        let mut stmt = conn.prepare("SELECT name FROM _migrations").map_err(|e| {
            AppError::new(
                "DB_MIGRATIONS_QUERY_FAILED",
                "Failed to query applied migrations",
            )
            .with_details(e.to_string())
        })?;

        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| {
                AppError::new(
                    "DB_MIGRATIONS_QUERY_FAILED",
                    "Failed to read applied migrations",
                )
                .with_details(e.to_string())
            })?;

        let mut set = HashSet::new();
        for r in rows {
            let name = r.map_err(|e| {
                AppError::new(
                    "DB_MIGRATIONS_QUERY_FAILED",
                    "Failed to read applied migration row",
                )
                .with_details(e.to_string())
            })?;
            set.insert(name);
        }
        set
    };

    for (name, sql) in migrations() {
        if applied.contains(name) {
            continue;
        }

        let tx = conn.transaction().map_err(|e| {
            AppError::new("DB_TX_FAILED", "Failed to start migration transaction")
                .with_details(e.to_string())
        })?;

        tx.execute_batch(sql).map_err(|e| {
            AppError::new("DB_MIGRATION_FAILED", format!("Migration {name} failed"))
                .with_details(e.to_string())
        })?;

        // Use SQLite to record the timestamp; this is operational metadata only.
        tx.execute(
      "INSERT INTO _migrations(name, applied_at) VALUES (?1, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
      [name],
    )
    .map_err(|e| {
      AppError::new("DB_MIGRATION_FAILED", format!("Failed to record migration {name}"))
        .with_details(e.to_string())
    })?;

        tx.commit().map_err(|e| {
            AppError::new("DB_TX_FAILED", "Failed to commit migration transaction")
                .with_details(e.to_string())
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::OptionalExtension;

    #[test]
    fn migrations_create_expected_tables() {
        let mut conn = open_in_memory().expect("open");
        migrate(&mut conn).expect("migrate");

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='incidents'")
            .unwrap();
        let name: Option<String> = stmt.query_row([], |row| row.get(0)).optional().unwrap();
        assert_eq!(name.as_deref(), Some("incidents"));
    }
}
