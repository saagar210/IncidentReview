pub mod chunking;
pub mod model;
pub mod store;

pub use model::{
    Citation, CitationLocator, EvidenceChunk, EvidenceChunkMeta, EvidenceChunkSummary, EvidenceOrigin,
    EvidenceSource, EvidenceSourceType, EvidenceTimeRange,
};
pub use store::{BuildChunksResult, EvidenceAddSourceInput, EvidenceQueryStore, EvidenceStore};

