use std::fs;
use std::path::{Path, PathBuf};

use qir_core::error::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceChunk {
    pub id: String,
    pub source: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct EvidenceStore {
    root: PathBuf,
}

impl EvidenceStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn chunks_dir(&self) -> PathBuf {
        self.root.join("chunks")
    }

    pub fn ensure_dirs(&self) -> Result<(), AppError> {
        fs::create_dir_all(self.chunks_dir()).map_err(|e| {
            AppError::new(
                "AI_EVIDENCE_STORE_FAILED",
                "Failed to create evidence store directories",
            )
            .with_details(e.to_string())
        })
    }

    pub fn put_chunk(&self, source: &str, text: &str) -> Result<EvidenceChunk, AppError> {
        self.ensure_dirs()?;

        let payload = format!("source={}\ntext={}", source, text);
        let digest = Sha256::digest(payload.as_bytes());
        let id = hex::encode(digest);

        let chunk = EvidenceChunk {
            id: id.clone(),
            source: source.to_string(),
            text: text.to_string(),
        };

        let path = self.chunks_dir().join(format!("{id}.json"));
        let json = serde_json::to_string_pretty(&chunk).map_err(|e| {
            AppError::new(
                "AI_EVIDENCE_STORE_FAILED",
                "Failed to serialize evidence chunk",
            )
            .with_details(e.to_string())
        })?;

        // Deterministic: chunk content is stable; overwriting is idempotent because ID is content-derived.
        fs::write(&path, json).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to write evidence chunk")
                .with_details(e.to_string())
        })?;

        Ok(chunk)
    }

    pub fn get_chunk(&self, id: &str) -> Result<EvidenceChunk, AppError> {
        let path = self.chunks_dir().join(format!("{id}.json"));
        let raw = fs::read_to_string(&path).map_err(|e| {
            AppError::new("AI_EVIDENCE_NOT_FOUND", "Evidence chunk not found")
                .with_details(format!("id={id}; err={e}"))
        })?;
        serde_json::from_str(&raw).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to parse evidence chunk")
                .with_details(e.to_string())
        })
    }
}

pub fn default_evidence_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("evidence")
}
