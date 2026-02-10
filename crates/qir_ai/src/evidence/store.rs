use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use qir_core::error::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::chunking::{build_chunks_for_source, ChunkDraft};
use super::model::{
    Citation, CitationLocator, EvidenceChunk, EvidenceChunkSummary, EvidenceContextResponse,
    EvidenceOrigin, EvidenceSource, EvidenceSourceType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceAddSourceInput {
    #[serde(rename = "type")]
    pub source_type: EvidenceSourceType,
    pub origin: EvidenceOrigin,
    pub label: String,
    pub created_at: String,
    // Only for paste-based sources.
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildChunksResult {
    pub source_id: Option<String>,
    pub chunk_count: u32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceQueryStore {
    pub include_text: bool,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EvidenceSourceRecord {
    pub source: EvidenceSource,
    pub content_rel_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EvidenceStore {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvidenceChunkSummaryRecord {
    pub summary: EvidenceChunkSummary,
    pub snippet: String,
}

impl EvidenceStore {
    pub fn open(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    fn sources_path(&self) -> PathBuf {
        self.root.join("sources.json")
    }

    fn sources_dir(&self) -> PathBuf {
        self.root.join("sources")
    }

    fn chunks_dir(&self) -> PathBuf {
        self.root.join("chunks")
    }

    fn chunk_summaries_dir(&self) -> PathBuf {
        self.root.join("chunk_summaries")
    }

    fn chunks_by_source_path(&self) -> PathBuf {
        self.root.join("chunks_by_source.json")
    }

    pub fn ensure_dirs(&self) -> Result<(), AppError> {
        fs::create_dir_all(self.root.as_path()).map_err(|e| {
            AppError::new(
                "AI_EVIDENCE_STORE_FAILED",
                "Failed to create evidence store directory",
            )
            .with_details(format!("path={}; err={}", self.root.display(), e))
        })?;
        fs::create_dir_all(self.sources_dir()).map_err(|e| {
            AppError::new(
                "AI_EVIDENCE_STORE_FAILED",
                "Failed to create evidence sources directory",
            )
            .with_details(format!("path={}; err={}", self.sources_dir().display(), e))
        })?;
        fs::create_dir_all(self.chunks_dir()).map_err(|e| {
            AppError::new(
                "AI_EVIDENCE_STORE_FAILED",
                "Failed to create evidence chunks directory",
            )
            .with_details(format!("path={}; err={}", self.chunks_dir().display(), e))
        })?;
        fs::create_dir_all(self.chunk_summaries_dir()).map_err(|e| {
            AppError::new(
                "AI_EVIDENCE_STORE_FAILED",
                "Failed to create evidence chunk summaries directory",
            )
            .with_details(format!(
                "path={}; err={}",
                self.chunk_summaries_dir().display(),
                e
            ))
        })?;
        Ok(())
    }

    fn read_sources(&self) -> Result<Vec<EvidenceSourceRecord>, AppError> {
        let path = self.sources_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let bytes = fs::read(&path).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to read evidence sources")
                .with_details(format!("path={}; err={}", path.display(), e))
        })?;
        serde_json::from_slice(&bytes).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to decode evidence sources")
                .with_details(format!("path={}; err={}", path.display(), e))
        })
    }

    fn write_sources(&self, records: &[EvidenceSourceRecord]) -> Result<(), AppError> {
        let path = self.sources_path();
        let tmp = path.with_extension("tmp");
        let json = serde_json::to_string_pretty(records).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to encode evidence sources")
                .with_details(e.to_string())
        })?;
        fs::write(&tmp, json.as_bytes()).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to write evidence sources")
                .with_details(format!("path={}; err={}", tmp.display(), e))
        })?;
        fs::rename(&tmp, &path).map_err(|e| {
            AppError::new(
                "AI_EVIDENCE_STORE_FAILED",
                "Failed to finalize evidence sources write",
            )
            .with_details(format!("tmp={}; dest={}; err={}", tmp.display(), path.display(), e))
        })?;
        Ok(())
    }

    fn normalize_descriptor_for_id(input: &EvidenceAddSourceInput) -> Result<String, AppError> {
        // Deterministic, minimal descriptor. created_at and label are not included.
        // origin.path is included when present.
        let v = serde_json::json!({
            "type": &input.source_type,
            "origin": {
                "kind": &input.origin.kind,
                "path": &input.origin.path,
            },
        });
        canonical_json_string(&v)
    }

    pub fn add_source(&self, input: EvidenceAddSourceInput) -> Result<EvidenceSource, AppError> {
        self.ensure_dirs()?;

        if input.label.trim().is_empty() {
            return Err(AppError::new(
                "AI_EVIDENCE_SOURCE_INVALID",
                "Evidence source label is required",
            ));
        }

        let origin_kind = input.origin.kind.as_str();
        match origin_kind {
            "file" | "directory" | "paste" => {}
            _ => {
                return Err(AppError::new(
                    "AI_EVIDENCE_SOURCE_INVALID",
                    "Evidence origin kind must be file, directory, or paste",
                )
                .with_details(format!("kind={}", input.origin.kind)));
            }
        }
        if (origin_kind == "file" || origin_kind == "directory") && input.origin.path.is_none() {
            return Err(AppError::new(
                "AI_EVIDENCE_SOURCE_INVALID",
                "Evidence origin path is required for file/directory sources",
            ));
        }
        if origin_kind == "paste" && input.text.as_deref().unwrap_or("").trim().is_empty() {
            return Err(AppError::new(
                "AI_EVIDENCE_SOURCE_INVALID",
                "Evidence paste text is required",
            ));
        }

        let descriptor = Self::normalize_descriptor_for_id(&input)?;
        let source_id = sha256_hex(descriptor.as_bytes());

        let mut origin = input.origin.clone();
        if let Some(p) = origin.path.as_ref() {
            // Normalize path separators/line endings are handled at chunk time.
            origin.path = Some(p.to_string());
        }

        let source = EvidenceSource {
            source_id: source_id.clone(),
            source_type: input.source_type.clone(),
            origin,
            label: input.label,
            created_at: input.created_at,
        };

        // Persist paste content deterministically under sources/<source_id>.txt
        let mut content_rel_path: Option<String> = None;
        if input.origin.kind == "paste" {
            let rel = format!("sources/{source_id}.txt");
            let abs = self.root.join(&rel);
            let normalized = normalize_text(input.text.as_deref().unwrap_or(""));
            fs::write(&abs, normalized.as_bytes()).map_err(|e| {
                AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to write paste evidence content")
                    .with_details(format!("path={}; err={}", abs.display(), e))
            })?;
            content_rel_path = Some(rel);
        }

        let mut records = self.read_sources()?;
        records.retain(|r| r.source.source_id != source_id);
        records.push(EvidenceSourceRecord {
            source: source.clone(),
            content_rel_path,
        });

        // Stable ordering for deterministic lists.
        records.sort_by(|a, b| a.source.source_id.cmp(&b.source.source_id));
        self.write_sources(&records)?;

        Ok(source)
    }

    pub fn list_sources(&self) -> Result<Vec<EvidenceSource>, AppError> {
        self.ensure_dirs()?;
        let mut records = self.read_sources()?;
        records.sort_by(|a, b| a.source.source_id.cmp(&b.source.source_id));
        Ok(records.into_iter().map(|r| r.source).collect())
    }

    fn read_chunks_by_source(&self) -> Result<BTreeMap<String, Vec<String>>, AppError> {
        let path = self.chunks_by_source_path();
        if !path.exists() {
            return Ok(BTreeMap::new());
        }
        let bytes = fs::read(&path).map_err(|e| {
            AppError::new(
                "AI_EVIDENCE_STORE_FAILED",
                "Failed to read chunks_by_source mapping",
            )
            .with_details(format!("path={}; err={}", path.display(), e))
        })?;
        serde_json::from_slice(&bytes).map_err(|e| {
            AppError::new(
                "AI_EVIDENCE_STORE_FAILED",
                "Failed to decode chunks_by_source mapping",
            )
            .with_details(format!("path={}; err={}", path.display(), e))
        })
    }

    fn write_chunks_by_source(&self, map: &BTreeMap<String, Vec<String>>) -> Result<(), AppError> {
        let path = self.chunks_by_source_path();
        let tmp = path.with_extension("tmp");
        let json = serde_json::to_string_pretty(map).map_err(|e| {
            AppError::new(
                "AI_EVIDENCE_STORE_FAILED",
                "Failed to encode chunks_by_source mapping",
            )
            .with_details(e.to_string())
        })?;
        fs::write(&tmp, json.as_bytes()).map_err(|e| {
            AppError::new(
                "AI_EVIDENCE_STORE_FAILED",
                "Failed to write chunks_by_source mapping",
            )
            .with_details(format!("path={}; err={}", tmp.display(), e))
        })?;
        fs::rename(&tmp, &path).map_err(|e| {
            AppError::new(
                "AI_EVIDENCE_STORE_FAILED",
                "Failed to finalize chunks_by_source mapping write",
            )
            .with_details(format!("tmp={}; dest={}; err={}", tmp.display(), path.display(), e))
        })?;
        Ok(())
    }

    fn chunk_path(&self, chunk_id: &str) -> PathBuf {
        self.chunks_dir().join(format!("{chunk_id}.json"))
    }

    fn chunk_summary_path(&self, chunk_id: &str) -> PathBuf {
        self.chunk_summaries_dir().join(format!("{chunk_id}.json"))
    }

    fn write_chunk(&self, chunk: &EvidenceChunk) -> Result<(), AppError> {
        let path = self.chunk_path(&chunk.chunk_id);
        let json = serde_json::to_string_pretty(chunk).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to encode evidence chunk")
                .with_details(e.to_string())
        })?;
        fs::write(&path, json.as_bytes()).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to write evidence chunk")
                .with_details(format!("path={}; err={}", path.display(), e))
        })?;
        Ok(())
    }

    fn write_chunk_summary(&self, chunk: &EvidenceChunk) -> Result<(), AppError> {
        let path = self.chunk_summary_path(&chunk.chunk_id);
        let summary = EvidenceChunkSummary {
            chunk_id: chunk.chunk_id.clone(),
            source_id: chunk.source_id.clone(),
            ordinal: chunk.ordinal,
            text_sha256: chunk.text_sha256.clone(),
            token_count_est: chunk.token_count_est,
            meta: chunk.meta.clone(),
        };
        let rec = EvidenceChunkSummaryRecord {
            summary,
            snippet: snippet_first_chars(&chunk.text, 280),
        };
        let json = serde_json::to_string_pretty(&rec).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to encode evidence chunk summary")
                .with_details(e.to_string())
        })?;
        fs::write(&path, json.as_bytes()).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to write evidence chunk summary")
                .with_details(format!("path={}; err={}", path.display(), e))
        })?;
        Ok(())
    }

    pub fn get_chunk(&self, chunk_id: &str) -> Result<EvidenceChunk, AppError> {
        self.ensure_dirs()?;
        let path = self.chunk_path(chunk_id);
        let raw = fs::read_to_string(&path).map_err(|e| {
            AppError::new("AI_EVIDENCE_NOT_FOUND", "Evidence chunk not found")
                .with_details(format!("id={chunk_id}; err={e}"))
        })?;
        serde_json::from_str(&raw).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to decode evidence chunk")
                .with_details(format!("path={}; err={}", path.display(), e))
        })
    }

    fn get_chunk_summary_record(&self, chunk_id: &str) -> Result<EvidenceChunkSummaryRecord, AppError> {
        self.ensure_dirs()?;
        let path = self.chunk_summary_path(chunk_id);
        if path.exists() {
            let raw = fs::read_to_string(&path).map_err(|e| {
                AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to read evidence chunk summary")
                    .with_details(format!("id={chunk_id}; err={e}"))
            })?;
            return serde_json::from_str(&raw).map_err(|e| {
                AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to decode evidence chunk summary")
                    .with_details(format!("path={}; err={}", path.display(), e))
            });
        }

        // Back-compat: if summaries haven't been written yet, derive deterministically from the chunk file.
        let chunk = self.get_chunk(chunk_id)?;
        self.write_chunk_summary(&chunk)?;
        Ok(EvidenceChunkSummaryRecord {
            summary: EvidenceChunkSummary {
                chunk_id: chunk.chunk_id,
                source_id: chunk.source_id,
                ordinal: chunk.ordinal,
                text_sha256: chunk.text_sha256,
                token_count_est: chunk.token_count_est,
                meta: chunk.meta,
            },
            snippet: snippet_first_chars(&chunk.text, 280),
        })
    }

    pub fn get_chunk_summary(&self, chunk_id: &str) -> Result<EvidenceChunkSummary, AppError> {
        Ok(self.get_chunk_summary_record(chunk_id)?.summary)
    }

    pub fn get_chunk_snippet(&self, chunk_id: &str) -> Result<String, AppError> {
        Ok(self.get_chunk_summary_record(chunk_id)?.snippet)
    }

    pub fn get_context(&self, chunk_id: &str, window: u32) -> Result<EvidenceContextResponse, AppError> {
        self.ensure_dirs()?;
        if window > 50 {
            return Err(AppError::new(
                "AI_EVIDENCE_CONTEXT_INVALID",
                "Context window too large",
            )
            .with_details(format!("window={window}")));
        }

        let center_rec = self.get_chunk_summary_record(chunk_id)?;
        let center = center_rec.summary;
        let map = self.read_chunks_by_source()?;
        let ids = map.get(&center.source_id).ok_or_else(|| {
            AppError::new("AI_EVIDENCE_NOT_FOUND", "Evidence chunk not found")
                .with_details(format!("id={chunk_id}; source_id={}", center.source_id))
        })?;

        let mut ordered: Vec<(u32, String)> = Vec::new();
        for cid in ids {
            let rec = self.get_chunk_summary_record(cid)?;
            ordered.push((rec.summary.ordinal, rec.summary.chunk_id));
        }
        ordered.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        let pos = ordered
            .iter()
            .position(|(_, id)| id.as_str() == chunk_id)
            .ok_or_else(|| {
                AppError::new("AI_EVIDENCE_NOT_FOUND", "Evidence chunk not found")
                    .with_details(format!("id={chunk_id}; source_id={}", center.source_id))
            })?;

        let w = window as usize;
        let start = pos.saturating_sub(w);
        let end = std::cmp::min(ordered.len(), pos + w + 1);

        let mut chunks: Vec<EvidenceChunkSummary> = Vec::new();
        for (_, cid) in ordered[start..end].iter() {
            let rec = self.get_chunk_summary_record(cid)?;
            chunks.push(rec.summary);
        }

        // Stable ordering: source_id asc, ordinal asc, chunk_id asc.
        chunks.sort_by(|a, b| {
            a.source_id
                .cmp(&b.source_id)
                .then(a.ordinal.cmp(&b.ordinal))
                .then(a.chunk_id.cmp(&b.chunk_id))
        });

        Ok(EvidenceContextResponse {
            center_chunk_id: center.chunk_id,
            chunks,
        })
    }

    fn delete_chunks(&self, chunk_ids: &[String]) -> Result<(), AppError> {
        for id in chunk_ids {
            let path = self.chunk_path(id);
            if path.exists() {
                fs::remove_file(&path).map_err(|e| {
                    AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to delete chunk file")
                        .with_details(format!("path={}; err={}", path.display(), e))
                })?;
            }
            let spath = self.chunk_summary_path(id);
            if spath.exists() {
                fs::remove_file(&spath).map_err(|e| {
                    AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to delete chunk summary file")
                        .with_details(format!("path={}; err={}", spath.display(), e))
                })?;
            }
        }
        Ok(())
    }

    fn read_source_record(&self, source_id: &str) -> Result<EvidenceSourceRecord, AppError> {
        let records = self.read_sources()?;
        records
            .into_iter()
            .find(|r| r.source.source_id == source_id)
            .ok_or_else(|| {
                AppError::new("AI_EVIDENCE_SOURCE_INVALID", "Evidence source not found")
                    .with_details(format!("source_id={source_id}"))
            })
    }

    fn load_paste_text(&self, rec: &EvidenceSourceRecord) -> Result<String, AppError> {
        let rel = rec.content_rel_path.as_ref().ok_or_else(|| {
            AppError::new(
                "AI_EVIDENCE_SOURCE_INVALID",
                "Paste evidence missing persisted content",
            )
        })?;
        let abs = self.root.join(rel);
        fs::read_to_string(&abs).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to read paste evidence content")
                .with_details(format!("path={}; err={}", abs.display(), e))
        })
    }

    pub fn build_chunks(
        &self,
        source_id: Option<String>,
        updated_at: &str,
    ) -> Result<BuildChunksResult, AppError> {
        self.ensure_dirs()?;

        let sources = if let Some(id) = source_id.as_deref() {
            vec![self.read_source_record(id)?]
        } else {
            self.read_sources()?
        };
        if sources.is_empty() {
            return Err(AppError::new(
                "AI_EVIDENCE_EMPTY",
                "No evidence sources available",
            ));
        }

        let mut chunks_by_source = self.read_chunks_by_source()?;
        let mut total: u32 = 0;

        for rec in sources {
            // Delete old chunks for this source to avoid stale data.
            if let Some(old) = chunks_by_source.get(&rec.source.source_id) {
                self.delete_chunks(old)?;
            }

            let drafts = build_chunks_for_source(self, &rec)?;
            let mut chunk_ids = Vec::new();
            for d in drafts {
                let chunk = self.chunk_from_draft(&rec.source.source_id, d)?;
                self.write_chunk(&chunk)?;
                self.write_chunk_summary(&chunk)?;
                chunk_ids.push(chunk.chunk_id);
                total += 1;
            }
            chunks_by_source.insert(rec.source.source_id.clone(), chunk_ids);
        }

        self.write_chunks_by_source(&chunks_by_source)?;

        Ok(BuildChunksResult {
            source_id,
            chunk_count: total,
            updated_at: updated_at.to_string(),
        })
    }

    fn chunk_from_draft(&self, source_id: &str, d: ChunkDraft) -> Result<EvidenceChunk, AppError> {
        let text = normalize_text(&d.text);
        let text_sha256 = sha256_hex(text.as_bytes());
        let meta_json = serde_json::to_value(&d.meta).map_err(|e| {
            AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to encode chunk meta")
                .with_details(e.to_string())
        })?;
        let meta_canon = canonical_json_string(&meta_json)?;
        let meta_sha256 = sha256_hex(meta_canon.as_bytes());

        let id_input = format!(
            "v1|{}|{}|{}|{}",
            source_id, d.ordinal, text_sha256, meta_sha256
        );
        let chunk_id = sha256_hex(id_input.as_bytes());

        Ok(EvidenceChunk {
            chunk_id,
            source_id: source_id.to_string(),
            ordinal: d.ordinal,
            text: text.clone(),
            text_sha256,
            token_count_est: d.token_count_est,
            meta: d.meta,
        })
    }

    pub fn list_chunks(
        &self,
        query: EvidenceQueryStore,
    ) -> Result<Vec<EvidenceChunkSummary>, AppError> {
        self.ensure_dirs()?;
        let map = self.read_chunks_by_source()?;
        let mut out: Vec<EvidenceChunkSummary> = Vec::new();

        let source_ids: Vec<String> = if let Some(id) = query.source_id.as_ref() {
            vec![id.clone()]
        } else {
            map.keys().cloned().collect()
        };

        for sid in source_ids {
            let ids = match map.get(&sid) {
                Some(v) => v.clone(),
                None => continue,
            };
            for cid in ids {
                let rec = self.get_chunk_summary_record(&cid)?;
                out.push(rec.summary);
            }
        }

        // Stable ordering: source_id asc, ordinal asc, chunk_id asc.
        out.sort_by(|a, b| {
            a.source_id
                .cmp(&b.source_id)
                .then(a.ordinal.cmp(&b.ordinal))
                .then(a.chunk_id.cmp(&b.chunk_id))
        });

        Ok(out)
    }

    pub fn citation_for_chunk(&self, chunk: &EvidenceChunk) -> Citation {
        Citation {
            chunk_id: chunk.chunk_id.clone(),
            locator: CitationLocator {
                source_id: chunk.source_id.clone(),
                ordinal: chunk.ordinal,
                text_sha256: chunk.text_sha256.clone(),
                char_range: None,
            },
        }
    }

    pub fn citation_for_summary(&self, chunk: &EvidenceChunkSummary) -> Citation {
        Citation {
            chunk_id: chunk.chunk_id.clone(),
            locator: CitationLocator {
                source_id: chunk.source_id.clone(),
                ordinal: chunk.ordinal,
                text_sha256: chunk.text_sha256.clone(),
                char_range: None,
            },
        }
    }

    pub fn validate_citations(
        &self,
        citations: &[Citation],
    ) -> Result<(), AppError> {
        if citations.is_empty() {
            return Err(AppError::new(
                "AI_CITATION_REQUIRED",
                "At least one citation is required",
            ));
        }
        for c in citations {
            let chunk = self.get_chunk(&c.chunk_id)?;
            let expected = self.citation_for_chunk(&chunk);
            if expected.locator != c.locator {
                return Err(AppError::new(
                    "AI_CITATION_INVALID",
                    "Citation locator does not match stored chunk metadata",
                )
                .with_details(format!(
                    "chunk_id={}; expected={:?}; got={:?}",
                    c.chunk_id, expected.locator, c.locator
                )));
            }
        }
        Ok(())
    }

    pub(crate) fn read_source_text_for_chunking(&self, rec: &EvidenceSourceRecord) -> Result<String, AppError> {
        if rec.source.origin.kind == "paste" {
            return self.load_paste_text(rec);
        }
        let path = rec
            .source
            .origin
            .path
            .as_ref()
            .ok_or_else(|| AppError::new("AI_EVIDENCE_SOURCE_INVALID", "Evidence source path missing"))?;
        let abs = PathBuf::from(path);
        if !abs.exists() {
            return Err(AppError::new("AI_EVIDENCE_SOURCE_INVALID", "Evidence source path does not exist")
                .with_details(format!("path={}", abs.display())));
        }
        if abs.is_dir() {
            return Err(AppError::new(
                "AI_EVIDENCE_SOURCE_INVALID",
                "Evidence text source must be a file (directory source is parsed by type-specific handlers)",
            )
            .with_details(format!("path={}", abs.display())));
        }
        fs::read_to_string(&abs).map_err(|e| {
            AppError::new("AI_EVIDENCE_SOURCE_INVALID", "Failed to read evidence file")
                .with_details(format!("path={}; err={}", abs.display(), e))
        })
    }
}

pub(crate) fn normalize_text(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

fn snippet_first_chars(text: &str, max_chars: usize) -> String {
    let t = text.trim();
    if t.len() <= max_chars {
        return t.to_string();
    }
    let mut s = t[..max_chars].to_string();
    s.push_str("...");
    s
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}

fn canonical_json_string(v: &serde_json::Value) -> Result<String, AppError> {
    let canon = canonicalize_json_value(v);
    serde_json::to_string(&canon).map_err(|e| {
        AppError::new("AI_EVIDENCE_STORE_FAILED", "Failed to canonicalize JSON")
            .with_details(e.to_string())
    })
}

fn canonicalize_json_value(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for k in keys {
                out.insert(k.clone(), canonicalize_json_value(&map[k]));
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(canonicalize_json_value).collect())
        }
        _ => v.clone(),
    }
}
