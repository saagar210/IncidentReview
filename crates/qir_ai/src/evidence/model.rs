use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSourceType {
    SanitizedExport,
    SlackTranscript,
    IncidentReportMd,
    FreeformText,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceOrigin {
    // file|directory|paste
    pub kind: String,
    // Absolute path when applicable.
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceSource {
    pub source_id: String,
    #[serde(rename = "type")]
    pub source_type: EvidenceSourceType,
    pub origin: EvidenceOrigin,
    pub label: String,
    pub created_at: String, // RFC3339
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceTimeRange {
    pub start_ts: Option<String>,
    pub end_ts: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceChunkMeta {
    pub kind: String,
    pub incident_keys: Option<Vec<String>>,
    pub time_range: Option<EvidenceTimeRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceChunk {
    pub chunk_id: String,
    pub source_id: String,
    pub ordinal: u32,
    pub text: String,
    pub text_sha256: String,
    pub token_count_est: u32,
    pub meta: EvidenceChunkMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceChunkSummary {
    pub chunk_id: String,
    pub source_id: String,
    pub ordinal: u32,
    pub text_sha256: String,
    pub token_count_est: u32,
    pub meta: EvidenceChunkMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceContextResponse {
    pub center_chunk_id: String,
    pub chunks: Vec<EvidenceChunkSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CitationLocator {
    pub source_id: String,
    pub ordinal: u32,
    pub text_sha256: String,
    pub char_range: Option<[u32; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Citation {
    pub chunk_id: String,
    pub locator: CitationLocator,
}
