use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use qir_core::error::AppError;
use serde::{Deserialize, Serialize};

use crate::embeddings::Embedder;

use super::store::EvidenceStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiIndexStatus {
    pub ready: bool,
    pub model: Option<String>,
    pub dims: Option<u32>,
    pub chunk_count: u32,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiIndexBuildInput {
    pub model: String,
    pub source_id: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct IndexStore {
    root: PathBuf,
}

impl IndexStore {
    pub fn open(root: PathBuf) -> Self {
        Self { root }
    }

    fn index_dir(&self) -> PathBuf {
        self.root.join("index")
    }

    fn status_path(&self) -> PathBuf {
        self.index_dir().join("index_status.json")
    }

    fn vectors_path(&self) -> PathBuf {
        self.index_dir().join("index_vectors.json")
    }

    fn ensure_dirs(&self) -> Result<(), AppError> {
        fs::create_dir_all(self.index_dir()).map_err(|e| {
            AppError::new("AI_INDEX_BUILD_FAILED", "Failed to create index directory")
                .with_details(format!("path={}; err={}", self.index_dir().display(), e))
        })
    }

    pub fn status(&self) -> Result<AiIndexStatus, AppError> {
        self.ensure_dirs()?;
        let path = self.status_path();
        if !path.exists() {
            return Ok(AiIndexStatus {
                ready: false,
                model: None,
                dims: None,
                chunk_count: 0,
                updated_at: None,
            });
        }
        let bytes = fs::read(&path).map_err(|e| {
            AppError::new("AI_INDEX_BUILD_FAILED", "Failed to read index status")
                .with_details(format!("path={}; err={}", path.display(), e))
        })?;
        serde_json::from_slice(&bytes).map_err(|e| {
            AppError::new("AI_INDEX_BUILD_FAILED", "Failed to decode index status")
                .with_details(format!("path={}; err={}", path.display(), e))
        })
    }

    fn write_status(&self, st: &AiIndexStatus) -> Result<(), AppError> {
        self.ensure_dirs()?;
        let path = self.status_path();
        let tmp = path.with_extension("tmp");
        let json = serde_json::to_string_pretty(st).map_err(|e| {
            AppError::new("AI_INDEX_BUILD_FAILED", "Failed to encode index status")
                .with_details(e.to_string())
        })?;
        fs::write(&tmp, json.as_bytes()).map_err(|e| {
            AppError::new("AI_INDEX_BUILD_FAILED", "Failed to write index status")
                .with_details(format!("path={}; err={}", tmp.display(), e))
        })?;
        fs::rename(&tmp, &path).map_err(|e| {
            AppError::new("AI_INDEX_BUILD_FAILED", "Failed to finalize index status write")
                .with_details(format!("tmp={}; dest={}; err={}", tmp.display(), path.display(), e))
        })?;
        Ok(())
    }

    fn write_vectors(&self, map: &BTreeMap<String, Vec<f32>>) -> Result<(), AppError> {
        self.ensure_dirs()?;
        let path = self.vectors_path();
        let tmp = path.with_extension("tmp");
        let json = serde_json::to_string_pretty(map).map_err(|e| {
            AppError::new("AI_INDEX_BUILD_FAILED", "Failed to encode index vectors")
                .with_details(e.to_string())
        })?;
        fs::write(&tmp, json.as_bytes()).map_err(|e| {
            AppError::new("AI_INDEX_BUILD_FAILED", "Failed to write index vectors")
                .with_details(format!("path={}; err={}", tmp.display(), e))
        })?;
        fs::rename(&tmp, &path).map_err(|e| {
            AppError::new("AI_INDEX_BUILD_FAILED", "Failed to finalize index vectors write")
                .with_details(format!("tmp={}; dest={}; err={}", tmp.display(), path.display(), e))
        })?;
        Ok(())
    }

    pub fn read_vectors(&self) -> Result<BTreeMap<String, Vec<f32>>, AppError> {
        self.ensure_dirs()?;
        let path = self.vectors_path();
        if !path.exists() {
            return Ok(BTreeMap::new());
        }
        let bytes = fs::read(&path).map_err(|e| {
            AppError::new("AI_INDEX_BUILD_FAILED", "Failed to read index vectors")
                .with_details(format!("path={}; err={}", path.display(), e))
        })?;
        serde_json::from_slice(&bytes).map_err(|e| {
            AppError::new("AI_INDEX_BUILD_FAILED", "Failed to decode index vectors")
                .with_details(format!("path={}; err={}", path.display(), e))
        })
    }

    pub fn build_with_embedder(
        &self,
        evidence: &EvidenceStore,
        embedder: &dyn Embedder,
        input: AiIndexBuildInput,
    ) -> Result<AiIndexStatus, AppError> {
        self.ensure_dirs()?;

        let chunk_summaries = evidence.list_chunks(super::store::EvidenceQueryStore {
            include_text: false,
            source_id: input.source_id.clone(),
        })?;
        if chunk_summaries.is_empty() {
            return Err(AppError::new(
                "AI_INDEX_NOT_READY",
                "No chunks available; build chunks before building the index",
            ));
        }

        // Stable order: chunk_id asc (ties deterministic).
        let mut ids = chunk_summaries
            .iter()
            .map(|c| c.chunk_id.clone())
            .collect::<Vec<_>>();
        ids.sort();

        let mut vectors: BTreeMap<String, Vec<f32>> = BTreeMap::new();
        let mut dims: Option<u32> = None;

        for chunk_id in ids.iter() {
            let chunk = evidence.get_chunk(chunk_id)?;
            let v = embedder.embed(&input.model, &chunk.text).map_err(|e| {
                AppError::new("AI_EMBEDDINGS_FAILED", "Failed to compute embeddings")
                    .with_details(format!("chunk_id={}; err={}", chunk_id, e))
                    .with_retryable(e.retryable)
            })?;
            let this_dims = v.len() as u32;
            if let Some(d) = dims {
                if d != this_dims {
                    return Err(AppError::new(
                        "AI_INDEX_BUILD_FAILED",
                        "Embedding dimension mismatch across chunks",
                    )
                    .with_details(format!("expected={}; got={}; chunk_id={}", d, this_dims, chunk_id)));
                }
            } else {
                dims = Some(this_dims);
            }
            vectors.insert(chunk_id.clone(), v);
        }

        self.write_vectors(&vectors)?;

        let st = AiIndexStatus {
            ready: true,
            model: Some(input.model),
            dims,
            chunk_count: vectors.len() as u32,
            updated_at: Some(input.updated_at),
        };
        self.write_status(&st)?;
        Ok(st)
    }
}
