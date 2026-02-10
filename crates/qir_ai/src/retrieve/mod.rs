use qir_core::error::AppError;
use serde::{Deserialize, Serialize};

use crate::embeddings::Embedder;
use crate::evidence::{Citation, EvidenceStore, IndexStore};

mod similarity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceQueryHit {
    pub chunk_id: String,
    pub source_id: String,
    pub score: f32,
    pub snippet: String,
    pub citation: Citation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceQueryResponse {
    pub hits: Vec<EvidenceQueryHit>,
}

pub fn query_with_embedder(
    evidence: &EvidenceStore,
    index: &IndexStore,
    embedder: &dyn Embedder,
    query: &str,
    top_k: u32,
    source_filter: Option<&[String]>,
) -> Result<EvidenceQueryResponse, AppError> {
    let q = query.trim();
    if q.is_empty() {
        return Err(AppError::new(
            "AI_RETRIEVAL_FAILED",
            "Query must not be empty",
        ));
    }
    let top_k = top_k.max(1).min(50);

    let st = index.status()?;
    if !st.ready {
        return Err(AppError::new(
            "AI_INDEX_NOT_READY",
            "Index not ready; build the index before querying",
        ));
    }
    let model = st.model.clone().ok_or_else(|| {
        AppError::new("AI_INDEX_NOT_READY", "Index status missing model")
    })?;
    let dims = st.dims.ok_or_else(|| AppError::new("AI_INDEX_NOT_READY", "Index status missing dims"))?;

    let qv = embedder.embed(&model, q)?;
    if qv.len() as u32 != dims {
        return Err(AppError::new(
            "AI_RETRIEVAL_FAILED",
            "Query embedding dims do not match index dims",
        )
        .with_details(format!("index_dims={dims}; query_dims={}", qv.len())));
    }

    let vectors = index.read_vectors()?;
    if vectors.is_empty() {
        return Err(AppError::new(
            "AI_INDEX_NOT_READY",
            "Index vectors missing; rebuild index",
        ));
    }

    let qnorm = similarity::l2_norm(&qv);
    if qnorm == 0.0 {
        return Err(AppError::new(
            "AI_RETRIEVAL_FAILED",
            "Query embedding norm is zero",
        ));
    }

    let mut hits: Vec<(String, f32)> = Vec::new();

    for (chunk_id, v) in vectors.iter() {
        if v.len() as u32 != dims {
            return Err(AppError::new(
                "AI_RETRIEVAL_FAILED",
                "Index vector dims mismatch",
            )
            .with_details(format!("chunk_id={chunk_id}; expected={dims}; got={}", v.len())));
        }

        if let Some(filter) = source_filter {
            let chunk = evidence.get_chunk(chunk_id)?;
            if !filter.iter().any(|sid| sid == &chunk.source_id) {
                continue;
            }
        }

        let vnorm = similarity::l2_norm(v);
        if vnorm == 0.0 {
            continue;
        }
        let score = similarity::cosine_similarity(&qv, v, qnorm, vnorm);
        hits.push((chunk_id.clone(), score));
    }

    hits.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });
    hits.truncate(top_k as usize);

    let mut out: Vec<EvidenceQueryHit> = Vec::new();
    for (chunk_id, score) in hits {
        let chunk = evidence.get_chunk(&chunk_id)?;
        let snippet = snippet_first_chars(&chunk.text, 280);
        out.push(EvidenceQueryHit {
            chunk_id: chunk.chunk_id.clone(),
            source_id: chunk.source_id.clone(),
            score,
            snippet,
            citation: evidence.citation_for_chunk(&chunk),
        });
    }

    Ok(EvidenceQueryResponse { hits: out })
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

