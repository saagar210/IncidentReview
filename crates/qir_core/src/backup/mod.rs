use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFileEntry {
    pub rel_path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCounts {
    pub incidents: i64,
    pub timeline_events: i64,
    pub artifacts_rows: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupDbInfo {
    pub filename: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupArtifactsInfo {
    pub included: bool,
    pub files: Vec<BackupFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub manifest_version: u32,
    pub app_version: String,
    pub export_time: String,
    pub schema_migrations: Vec<String>,
    pub counts: BackupCounts,
    pub db: BackupDbInfo,
    pub artifacts: BackupArtifactsInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCreateResult {
    pub backup_dir: String,
    pub manifest: BackupManifest,
}

fn sha256_file_hex(path: &Path) -> Result<(String, u64), AppError> {
    let mut f = fs::File::open(path).map_err(|e| {
        AppError::new("DB_BACKUP_FILE_OPEN_FAILED", "Failed to open file for hashing")
            .with_details(format!("path={}: {}", path.display(), e))
    })?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    let mut total: u64 = 0;
    loop {
        let n = f.read(&mut buf).map_err(|e| {
            AppError::new("DB_BACKUP_FILE_READ_FAILED", "Failed to read file for hashing")
                .with_details(format!("path={}: {}", path.display(), e))
        })?;
        if n == 0 {
            break;
        }
        total += n as u64;
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok((hex::encode(digest), total))
}

fn list_migration_names(conn: &Connection) -> Result<Vec<String>, AppError> {
    let mut stmt = conn
        .prepare("SELECT name FROM _migrations ORDER BY name ASC")
        .map_err(|e| {
            AppError::new("DB_BACKUP_MIGRATIONS_QUERY_FAILED", "Failed to query migrations")
                .with_details(e.to_string())
        })?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| {
            AppError::new("DB_BACKUP_MIGRATIONS_QUERY_FAILED", "Failed to read migrations")
                .with_details(e.to_string())
        })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| {
            AppError::new("DB_BACKUP_MIGRATIONS_QUERY_FAILED", "Failed to read migration row")
                .with_details(e.to_string())
        })?);
    }
    Ok(out)
}

fn query_counts(conn: &Connection) -> Result<BackupCounts, AppError> {
    let incidents: i64 = conn.query_row("SELECT COUNT(*) FROM incidents", [], |row| row.get(0))
        .map_err(|e| AppError::new("DB_BACKUP_COUNTS_FAILED", "Failed to count incidents").with_details(e.to_string()))?;
    let timeline_events: i64 = conn
        .query_row("SELECT COUNT(*) FROM timeline_events", [], |row| row.get(0))
        .map_err(|e| {
            AppError::new("DB_BACKUP_COUNTS_FAILED", "Failed to count timeline events")
                .with_details(e.to_string())
        })?;
    let artifacts_rows: i64 = conn
        .query_row("SELECT COUNT(*) FROM artifacts", [], |row| row.get(0))
        .map_err(|e| {
            AppError::new("DB_BACKUP_COUNTS_FAILED", "Failed to count artifacts rows")
                .with_details(e.to_string())
        })?;
    Ok(BackupCounts {
        incidents,
        timeline_events,
        artifacts_rows,
    })
}

fn filename_safe_timestamp(export_time: &str) -> String {
    // Keep backup dir names human-friendly but filesystem-safe and deterministic.
    // We do not attempt fuzzy parsing; this is just a deterministic normalization.
    export_time
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => c,
            _ => '_',
        })
        .collect()
}

fn list_files_recursive_sorted(root: &Path) -> Result<Vec<PathBuf>, AppError> {
    fn walk(dir: &Path, acc: &mut Vec<PathBuf>) -> Result<(), AppError> {
        let mut entries: Vec<fs::DirEntry> = fs::read_dir(dir)
            .map_err(|e| {
                AppError::new(
                    "DB_BACKUP_ARTIFACTS_READDIR_FAILED",
                    "Failed to read artifacts directory",
                )
                .with_details(format!("path={}: {}", dir.display(), e))
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AppError::new(
                    "DB_BACKUP_ARTIFACTS_READDIR_FAILED",
                    "Failed to read artifacts directory entry",
                )
                .with_details(format!("path={}: {}", dir.display(), e))
            })?;

        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let p = e.path();
            let meta = e.metadata().map_err(|err| {
                AppError::new(
                    "DB_BACKUP_ARTIFACTS_STAT_FAILED",
                    "Failed to stat artifacts entry",
                )
                .with_details(format!("path={}: {}", p.display(), err))
            })?;
            if meta.is_dir() {
                walk(&p, acc)?;
            } else if meta.is_file() {
                acc.push(p);
            }
        }
        Ok(())
    }

    let mut out = Vec::new();
    walk(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn copy_artifacts_dir(src: &Path, dst: &Path) -> Result<Vec<BackupFileEntry>, AppError> {
    let files = list_files_recursive_sorted(src)?;
    let mut manifest_files = Vec::new();

    for abs in files {
        let rel = abs.strip_prefix(src).map_err(|e| {
            AppError::new(
                "DB_BACKUP_ARTIFACTS_PATH_FAILED",
                "Failed to compute relative artifacts path",
            )
            .with_details(e.to_string())
        })?;
        let rel_str = rel.to_string_lossy().to_string();
        let target = dst.join(rel);

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::new(
                    "DB_BACKUP_ARTIFACTS_MKDIR_FAILED",
                    "Failed to create artifacts directory in backup",
                )
                .with_details(format!("path={}: {}", parent.display(), e))
            })?;
        }

        fs::copy(&abs, &target).map_err(|e| {
            AppError::new(
                "DB_BACKUP_ARTIFACTS_COPY_FAILED",
                "Failed to copy artifact into backup",
            )
            .with_details(format!("src={} dst={}: {}", abs.display(), target.display(), e))
        })?;

        let (sha, bytes) = sha256_file_hex(&target)?;
        manifest_files.push(BackupFileEntry {
            rel_path: rel_str,
            sha256: sha,
            bytes,
        });
    }

    manifest_files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Ok(manifest_files)
}

pub fn create_backup(
    conn: &Connection,
    db_path: &Path,
    artifacts_dir: Option<&Path>,
    destination_dir: &Path,
    export_time: &str,
    app_version: &str,
) -> Result<BackupCreateResult, AppError> {
    if !destination_dir.is_dir() {
        return Err(AppError::new(
            "DB_BACKUP_DEST_NOT_DIR",
            "Backup destination must be an existing directory",
        )
        .with_details(destination_dir.display().to_string()));
    }

    let folder_name = format!(
        "IncidentReviewBackup_{}",
        filename_safe_timestamp(export_time)
    );
    let backup_dir = destination_dir.join(folder_name);
    if backup_dir.exists() {
        return Err(AppError::new(
            "DB_BACKUP_DEST_EXISTS",
            "Backup destination folder already exists",
        )
        .with_details(backup_dir.display().to_string()));
    }
    fs::create_dir_all(&backup_dir).map_err(|e| {
        AppError::new("DB_BACKUP_MKDIR_FAILED", "Failed to create backup directory")
            .with_details(format!("path={}: {}", backup_dir.display(), e))
    })?;

    // Create a consistent SQLite snapshot via the backup API (avoids unsafe file copying).
    let dest_db_path = backup_dir.join("incidentreview.sqlite");
    let mut dest_conn = Connection::open(&dest_db_path).map_err(|e| {
        AppError::new("DB_BACKUP_DB_CREATE_FAILED", "Failed to create backup SQLite file")
            .with_details(e.to_string())
    })?;
    {
        let backup = rusqlite::backup::Backup::new(conn, &mut dest_conn).map_err(|e| {
            AppError::new("DB_BACKUP_DB_SNAPSHOT_FAILED", "Failed to start SQLite backup")
                .with_details(e.to_string())
        })?;
        backup
            .run_to_completion(5, std::time::Duration::from_millis(50), None)
            .map_err(|e| {
                AppError::new(
                    "DB_BACKUP_DB_SNAPSHOT_FAILED",
                    "Failed to complete SQLite backup",
                )
                .with_details(e.to_string())
            })?;
    }

    let (db_sha, db_bytes) = sha256_file_hex(&dest_db_path)?;

    // Copy artifacts directory if present.
    let artifacts_dst = backup_dir.join("artifacts");
    let (artifacts_included, artifacts_files) = match artifacts_dir {
        Some(dir) if dir.is_dir() => {
            fs::create_dir_all(&artifacts_dst).map_err(|e| {
                AppError::new(
                    "DB_BACKUP_ARTIFACTS_MKDIR_FAILED",
                    "Failed to create artifacts folder in backup",
                )
                .with_details(format!("path={}: {}", artifacts_dst.display(), e))
            })?;
            (true, copy_artifacts_dir(dir, &artifacts_dst)?)
        }
        _ => (false, Vec::new()),
    };

    let migrations = list_migration_names(conn)?;
    let counts = query_counts(conn)?;

    let manifest = BackupManifest {
        manifest_version: 1,
        app_version: app_version.to_string(),
        export_time: export_time.to_string(),
        schema_migrations: migrations,
        counts,
        db: BackupDbInfo {
            filename: "incidentreview.sqlite".to_string(),
            sha256: db_sha,
            bytes: db_bytes,
        },
        artifacts: BackupArtifactsInfo {
            included: artifacts_included,
            files: artifacts_files,
        },
    };

    let manifest_path = backup_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| {
        AppError::new(
            "DB_BACKUP_MANIFEST_ENCODE_FAILED",
            "Failed to encode backup manifest",
        )
        .with_details(e.to_string())
    })?;
    fs::write(&manifest_path, manifest_json.as_bytes()).map_err(|e| {
        AppError::new(
            "DB_BACKUP_MANIFEST_WRITE_FAILED",
            "Failed to write backup manifest",
        )
        .with_details(format!("path={}: {}", manifest_path.display(), e))
    })?;

    // Best-effort sanity: ensure the DB we backed up exists on disk.
    if !db_path.exists() {
        return Err(AppError::new(
            "DB_BACKUP_SOURCE_DB_MISSING",
            "Source DB path does not exist (unexpected)",
        )
        .with_details(db_path.display().to_string()));
    }

    Ok(BackupCreateResult {
        backup_dir: backup_dir.to_string_lossy().to_string(),
        manifest,
    })
}

pub fn read_manifest(backup_dir: &Path) -> Result<BackupManifest, AppError> {
    let path = backup_dir.join("manifest.json");
    let bytes = fs::read(&path).map_err(|e| {
        AppError::new(
            "DB_BACKUP_MANIFEST_READ_FAILED",
            "Failed to read backup manifest",
        )
        .with_details(format!("path={}: {}", path.display(), e))
    })?;
    serde_json::from_slice::<BackupManifest>(&bytes).map_err(|e| {
        AppError::new(
            "DB_BACKUP_MANIFEST_DECODE_FAILED",
            "Failed to decode backup manifest",
        )
        .with_details(e.to_string())
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub ok: bool,
    pub restored_db_path: String,
    pub restored_artifacts: bool,
}

pub fn restore_from_backup(
    backup_dir: &Path,
    target_db_path: &Path,
    target_artifacts_dir: Option<&Path>,
    allow_overwrite: bool,
) -> Result<RestoreResult, AppError> {
    let manifest = read_manifest(backup_dir)?;
    if manifest.manifest_version != 1 {
        return Err(AppError::new(
            "DB_RESTORE_UNSUPPORTED_MANIFEST",
            "Unsupported backup manifest version",
        )
        .with_details(format!("version={}", manifest.manifest_version)));
    }

    let backup_db_path = backup_dir.join(&manifest.db.filename);
    if !backup_db_path.is_file() {
        return Err(AppError::new(
            "DB_RESTORE_DB_MISSING",
            "Backup DB file is missing",
        )
        .with_details(backup_db_path.display().to_string()));
    }

    let (actual_sha, _bytes) = sha256_file_hex(&backup_db_path)?;
    if actual_sha != manifest.db.sha256 {
        return Err(AppError::new(
            "DB_RESTORE_DB_HASH_MISMATCH",
            "Backup DB hash does not match manifest",
        )
        .with_details(format!(
            "expected={} actual={}",
            manifest.db.sha256, actual_sha
        )));
    }

    if target_db_path.exists() && !allow_overwrite {
        return Err(AppError::new(
            "DB_RESTORE_CONFIRM_REQUIRED",
            "Restore requires explicit overwrite confirmation",
        )
        .with_details(target_db_path.display().to_string()));
    }

    if let Some(parent) = target_db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::new("DB_RESTORE_MKDIR_FAILED", "Failed to create target DB directory")
                .with_details(format!("path={}: {}", parent.display(), e))
        })?;
    }

    // Stage into a temp file in the target directory, then swap into place.
    let tmp_path = target_db_path.with_extension("restore_tmp");
    fs::copy(&backup_db_path, &tmp_path).map_err(|e| {
        AppError::new("DB_RESTORE_COPY_FAILED", "Failed to stage restore DB file")
            .with_details(format!("src={} dst={}: {}", backup_db_path.display(), tmp_path.display(), e))
    })?;

    // Keep a pre-restore copy to avoid data loss; user can delete manually if desired.
    if target_db_path.exists() {
        let pre = target_db_path.with_extension("pre_restore");
        fs::rename(target_db_path, &pre).map_err(|e| {
            AppError::new(
                "DB_RESTORE_SWAP_FAILED",
                "Failed to move existing DB out of the way",
            )
            .with_details(format!("src={} dst={}: {}", target_db_path.display(), pre.display(), e))
        })?;
    }
    fs::rename(&tmp_path, target_db_path).map_err(|e| {
        AppError::new("DB_RESTORE_SWAP_FAILED", "Failed to move restored DB into place")
            .with_details(format!("src={} dst={}: {}", tmp_path.display(), target_db_path.display(), e))
    })?;

    // Restore artifacts if present in backup and a target dir is configured.
    let mut restored_artifacts = false;
    if manifest.artifacts.included {
        if let Some(dst_root) = target_artifacts_dir {
            if dst_root.exists() && !allow_overwrite {
                return Err(AppError::new(
                    "DB_RESTORE_CONFIRM_REQUIRED",
                    "Restore requires explicit overwrite confirmation",
                )
                .with_details(dst_root.display().to_string()));
            }

            let backup_artifacts = backup_dir.join("artifacts");
            if !backup_artifacts.is_dir() {
                return Err(AppError::new(
                    "DB_RESTORE_ARTIFACTS_MISSING",
                    "Backup indicates artifacts included but folder is missing",
                )
                .with_details(backup_artifacts.display().to_string()));
            }

            // Stage artifacts into a temp folder then swap.
            let tmp_artifacts = dst_root.with_extension("restore_tmp");
            if tmp_artifacts.exists() {
                fs::remove_dir_all(&tmp_artifacts).map_err(|e| {
                    AppError::new(
                        "DB_RESTORE_ARTIFACTS_CLEAN_FAILED",
                        "Failed to clean existing temp artifacts dir",
                    )
                    .with_details(format!("path={}: {}", tmp_artifacts.display(), e))
                })?;
            }
            fs::create_dir_all(&tmp_artifacts).map_err(|e| {
                AppError::new(
                    "DB_RESTORE_MKDIR_FAILED",
                    "Failed to create temp artifacts dir",
                )
                .with_details(format!("path={}: {}", tmp_artifacts.display(), e))
            })?;

            // Copy artifacts according to manifest ordering and validate hashes.
            for f in &manifest.artifacts.files {
                let src = backup_artifacts.join(&f.rel_path);
                let dst = tmp_artifacts.join(&f.rel_path);
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        AppError::new(
                            "DB_RESTORE_MKDIR_FAILED",
                            "Failed to create artifacts subdirectory",
                        )
                        .with_details(format!("path={}: {}", parent.display(), e))
                    })?;
                }
                fs::copy(&src, &dst).map_err(|e| {
                    AppError::new(
                        "DB_RESTORE_COPY_FAILED",
                        "Failed to restore artifact file",
                    )
                    .with_details(format!("src={} dst={}: {}", src.display(), dst.display(), e))
                })?;

                let (sha, _bytes) = sha256_file_hex(&dst)?;
                if sha != f.sha256 {
                    return Err(AppError::new(
                        "DB_RESTORE_ARTIFACT_HASH_MISMATCH",
                        "Restored artifact hash mismatch",
                    )
                    .with_details(format!(
                        "file={} expected={} actual={}",
                        f.rel_path, f.sha256, sha
                    )));
                }
            }

            if dst_root.exists() {
                let pre = dst_root.with_extension("pre_restore");
                if pre.exists() {
                    fs::remove_dir_all(&pre).map_err(|e| {
                        AppError::new(
                            "DB_RESTORE_ARTIFACTS_CLEAN_FAILED",
                            "Failed to remove previous pre_restore artifacts dir",
                        )
                        .with_details(format!("path={}: {}", pre.display(), e))
                    })?;
                }
                fs::rename(dst_root, &pre).map_err(|e| {
                    AppError::new(
                        "DB_RESTORE_SWAP_FAILED",
                        "Failed to move existing artifacts out of the way",
                    )
                    .with_details(format!("src={} dst={}: {}", dst_root.display(), pre.display(), e))
                })?;
            }
            fs::rename(&tmp_artifacts, dst_root).map_err(|e| {
                AppError::new(
                    "DB_RESTORE_SWAP_FAILED",
                    "Failed to move restored artifacts into place",
                )
                .with_details(format!("src={} dst={}: {}", tmp_artifacts.display(), dst_root.display(), e))
            })?;

            restored_artifacts = true;
        }
    }

    Ok(RestoreResult {
        ok: true,
        restored_db_path: target_db_path.to_string_lossy().to_string(),
        restored_artifacts,
    })
}
