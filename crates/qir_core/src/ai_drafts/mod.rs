use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AiDraftSectionType {
    ExecSummary,
    IncidentHighlightsTopN,
    ThemeAnalysis,
    ActionPlanNextQuarter,
    QuarterNarrativeRecap,
}

impl AiDraftSectionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiDraftSectionType::ExecSummary => "exec_summary",
            AiDraftSectionType::IncidentHighlightsTopN => "incident_highlights_top_n",
            AiDraftSectionType::ThemeAnalysis => "theme_analysis",
            AiDraftSectionType::ActionPlanNextQuarter => "action_plan_next_quarter",
            AiDraftSectionType::QuarterNarrativeRecap => "quarter_narrative_recap",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "exec_summary" => Some(Self::ExecSummary),
            "incident_highlights_top_n" => Some(Self::IncidentHighlightsTopN),
            "theme_analysis" => Some(Self::ThemeAnalysis),
            "action_plan_next_quarter" => Some(Self::ActionPlanNextQuarter),
            "quarter_narrative_recap" => Some(Self::QuarterNarrativeRecap),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiDraftArtifact {
    pub id: i64,
    pub quarter_label: String,
    pub section_type: AiDraftSectionType,
    pub draft_text: String,
    pub citation_chunk_ids: Vec<String>,
    pub model_name: String,
    pub model_params_hash: String,
    pub prompt_template_version: String,
    pub created_at: String, // RFC3339 (operational metadata)
    pub artifact_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_draft_id: Option<i64>, // For revision chains
    #[serde(default)]
    pub revision_number: i32, // Which revision in the chain (1 = first/root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision_notes: Option<String>, // User notes on this revision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_label: Option<String>, // e.g., "Alternative A", "Alternative B"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateAiDraftInput {
    pub quarter_label: String,
    pub section_type: AiDraftSectionType,
    pub draft_text: String,
    pub citation_chunk_ids: Vec<String>,
    pub model_name: String,
    pub model_params_hash: String,
    pub prompt_template_version: String,
    pub created_at: String, // RFC3339
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_draft_id: Option<i64>, // For revision chains
    #[serde(default)]
    pub revision_notes: Option<String>, // User notes on revision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_label: Option<String>, // Label for alternative branches
}

/// Response containing a draft lineage (parent -> child chain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftLineage {
    pub root_id: i64,
    pub path: Vec<AiDraftArtifact>, // Root to current
    pub siblings: Vec<AiDraftArtifact>, // Alternatives at same level
    pub children: Vec<AiDraftArtifact>, // Next generation
}

#[derive(Debug, Clone, Serialize)]
struct HashPayload<'a> {
    quarter_label: &'a str,
    section_type: &'a str,
    draft_text: &'a str,
    citation_chunk_ids: &'a [String],
    model_name: &'a str,
    model_params_hash: &'a str,
    prompt_template_version: &'a str,
    created_at: &'a str,
}

fn compute_artifact_hash(input: &CreateAiDraftInput) -> Result<String, AppError> {
    let payload = HashPayload {
        quarter_label: input.quarter_label.as_str(),
        section_type: input.section_type.as_str(),
        draft_text: input.draft_text.as_str(),
        citation_chunk_ids: &input.citation_chunk_ids,
        model_name: input.model_name.as_str(),
        model_params_hash: input.model_params_hash.as_str(),
        prompt_template_version: input.prompt_template_version.as_str(),
        created_at: input.created_at.as_str(),
    };
    let json = serde_json::to_string(&payload).map_err(|e| {
        AppError::new("DB_AI_DRAFT_HASH_FAILED", "Failed to serialize AI draft hash payload")
            .with_details(e.to_string())
    })?;
    let digest = Sha256::digest(json.as_bytes());
    Ok(hex::encode(digest))
}

pub fn create_ai_draft(conn: &Connection, input: CreateAiDraftInput) -> Result<AiDraftArtifact, AppError> {
    if input.citation_chunk_ids.is_empty() {
        return Err(AppError::new(
            "AI_CITATION_REQUIRED",
            "At least one citation chunk_id is required to store a draft",
        ));
    }
    if input.quarter_label.trim().is_empty() {
        return Err(AppError::new(
            "DB_AI_DRAFT_INVALID",
            "quarter_label is required",
        ));
    }
    if input.draft_text.trim().is_empty() {
        return Err(AppError::new(
            "DB_AI_DRAFT_INVALID",
            "draft_text is required",
        ));
    }
    if input.model_name.trim().is_empty()
        || input.model_params_hash.trim().is_empty()
        || input.prompt_template_version.trim().is_empty()
    {
        return Err(AppError::new(
            "DB_AI_DRAFT_INVALID",
            "model metadata is required",
        ));
    }

    let artifact_hash = compute_artifact_hash(&input)?;
    let citation_chunk_ids_json = serde_json::to_string(&input.citation_chunk_ids).map_err(|e| {
        AppError::new(
            "DB_AI_DRAFT_INVALID",
            "Failed to encode citation chunk IDs",
        )
        .with_details(e.to_string())
    })?;

    // Compute revision number
    let revision_number = if let Some(parent_id) = input.parent_draft_id {
        // Get parent's revision number and increment
        get_ai_draft(conn, parent_id)?
            .map(|parent| parent.revision_number + 1)
            .unwrap_or(2) // Fallback to 2 if parent not found
    } else {
        1 // Root draft
    };

    conn.execute(
        r#"
        INSERT INTO ai_drafts(
          quarter_label, section_type, draft_text, citation_chunk_ids_json,
          model_name, model_params_hash, prompt_template_version, created_at, artifact_hash,
          parent_draft_id, revision_number, revision_notes, branch_label
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#,
        params![
            input.quarter_label,
            input.section_type.as_str(),
            input.draft_text,
            citation_chunk_ids_json,
            input.model_name,
            input.model_params_hash,
            input.prompt_template_version,
            input.created_at,
            artifact_hash,
            input.parent_draft_id,
            revision_number,
            input.revision_notes,
            input.branch_label,
        ],
    )
    .map_err(|e| {
        AppError::new("DB_AI_DRAFT_CREATE_FAILED", "Failed to store AI draft artifact")
            .with_details(e.to_string())
    })?;

    let id = conn.last_insert_rowid();
    get_ai_draft(conn, id)?.ok_or_else(|| {
        AppError::new(
            "DB_AI_DRAFT_CREATE_FAILED",
            "AI draft artifact stored but could not be read back",
        )
    })
}

pub fn list_ai_drafts(conn: &Connection, quarter_label: Option<&str>) -> Result<Vec<AiDraftArtifact>, AppError> {
    let mut out: Vec<AiDraftArtifact> = Vec::new();

    let mut stmt = if quarter_label.is_some() {
        conn.prepare(
            r#"
            SELECT id, quarter_label, section_type, draft_text, citation_chunk_ids_json,
                   model_name, model_params_hash, prompt_template_version, created_at, artifact_hash,
                   parent_draft_id, revision_number, revision_notes, branch_label
            FROM ai_drafts
            WHERE quarter_label = ?1
            ORDER BY created_at DESC, id DESC
            "#,
        )
    } else {
        conn.prepare(
            r#"
            SELECT id, quarter_label, section_type, draft_text, citation_chunk_ids_json,
                   model_name, model_params_hash, prompt_template_version, created_at, artifact_hash,
                   parent_draft_id, revision_number, revision_notes, branch_label
            FROM ai_drafts
            ORDER BY created_at DESC, id DESC
            "#,
        )
    }
    .map_err(|e| {
        AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to query AI draft artifacts")
            .with_details(e.to_string())
    })?;

    let row_mapper = |row: &rusqlite::Row<'_>| -> Result<AiDraftArtifact, rusqlite::Error> {
        let section_type_raw: String = row.get(2)?;
        let section_type = AiDraftSectionType::from_str(&section_type_raw).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid section_type",
                )),
            )
        })?;

        let citation_chunk_ids_json: String = row.get(4)?;
        let citation_chunk_ids: Vec<String> = serde_json::from_str(&citation_chunk_ids_json)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

        Ok(AiDraftArtifact {
            id: row.get(0)?,
            quarter_label: row.get(1)?,
            section_type,
            draft_text: row.get(3)?,
            citation_chunk_ids,
            model_name: row.get(5)?,
            model_params_hash: row.get(6)?,
            prompt_template_version: row.get(7)?,
            created_at: row.get(8)?,
            artifact_hash: row.get(9)?,
            parent_draft_id: row.get(10)?,
            revision_number: row.get(11)?,
            revision_notes: row.get(12)?,
            branch_label: row.get(13)?,
        })
    };

    let rows = if let Some(q) = quarter_label {
        stmt.query_map([q], row_mapper)
    } else {
        stmt.query_map([], row_mapper)
    }
    .map_err(|e| {
        AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to map AI draft artifacts")
            .with_details(e.to_string())
    })?;

    for r in rows {
        out.push(r.map_err(|e| {
            AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to read AI draft artifact row")
                .with_details(e.to_string())
        })?);
    }

    Ok(out)
}

pub fn get_ai_draft(conn: &Connection, id: i64) -> Result<Option<AiDraftArtifact>, AppError> {
    let mut stmt = conn
        .prepare(
            r#"
        SELECT id, quarter_label, section_type, draft_text, citation_chunk_ids_json,
               model_name, model_params_hash, prompt_template_version, created_at, artifact_hash,
               parent_draft_id, revision_number, revision_notes, branch_label
        FROM ai_drafts
        WHERE id = ?1
        "#,
        )
        .map_err(|e| {
            AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to prepare AI draft query")
                .with_details(e.to_string())
        })?;

    let row = stmt
        .query_row([id], |row| {
            let section_type_raw: String = row.get(2)?;
            let section_type = AiDraftSectionType::from_str(&section_type_raw).ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid section_type")),
                )
            })?;
            let citation_chunk_ids_json: String = row.get(4)?;
            let citation_chunk_ids: Vec<String> =
                serde_json::from_str(&citation_chunk_ids_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;
            Ok(AiDraftArtifact {
                id: row.get(0)?,
                quarter_label: row.get(1)?,
                section_type,
                draft_text: row.get(3)?,
                citation_chunk_ids,
                model_name: row.get(5)?,
                model_params_hash: row.get(6)?,
                prompt_template_version: row.get(7)?,
                created_at: row.get(8)?,
                artifact_hash: row.get(9)?,
                parent_draft_id: row.get(10)?,
                revision_number: row.get(11)?,
                revision_notes: row.get(12)?,
                branch_label: row.get(13)?,
            })
        })
        .optional()
        .map_err(|e| {
            AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to query AI draft artifact")
                .with_details(e.to_string())
        })?;

    Ok(row)
}

/// Get the root draft ID for a given draft (by walking up parent chain)
pub fn get_root_draft_id(conn: &Connection, mut id: i64) -> Result<i64, AppError> {
    let mut visited = std::collections::HashSet::new();
    loop {
        if visited.contains(&id) {
            return Err(AppError::new(
                "DB_AI_DRAFT_INVALID",
                "Circular reference detected in draft parent chain",
            ));
        }
        visited.insert(id);

        let draft = get_ai_draft(conn, id)?
            .ok_or_else(|| AppError::new("DB_NOT_FOUND", "Draft not found"))?;

        if let Some(parent_id) = draft.parent_draft_id {
            id = parent_id;
        } else {
            return Ok(id); // Found root
        }
    }
}

/// Get the lineage (path from root to current draft)
pub fn get_draft_lineage(conn: &Connection, draft_id: i64) -> Result<DraftLineage, AppError> {
    let root_id = get_root_draft_id(conn, draft_id)?;
    let current = get_ai_draft(conn, draft_id)?
        .ok_or_else(|| AppError::new("DB_NOT_FOUND", "Draft not found"))?;

    // Walk from root to current
    let mut path = Vec::new();
    let mut current_id = root_id;
    let mut visited = std::collections::HashSet::new();

    loop {
        if visited.contains(&current_id) {
            return Err(AppError::new(
                "DB_AI_DRAFT_INVALID",
                "Circular reference in draft chain",
            ));
        }
        visited.insert(current_id);

        let draft = get_ai_draft(conn, current_id)?
            .ok_or_else(|| AppError::new("DB_NOT_FOUND", "Draft not found"))?;
        path.push(draft.clone());

        if current_id == draft_id {
            break;
        }

        // Find next in chain (child of current with same lineage)
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, quarter_label, section_type, draft_text, citation_chunk_ids_json,
                       model_name, model_params_hash, prompt_template_version, created_at, artifact_hash,
                       parent_draft_id, revision_number, revision_notes, branch_label
                FROM ai_drafts
                WHERE parent_draft_id = ?1 AND revision_number = ?2
                LIMIT 1
                "#,
            )
            .map_err(|e| {
                AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to query draft lineage")
                    .with_details(e.to_string())
            })?;

        let next_id = stmt
            .query_row([current_id, (draft.revision_number + 1) as i64], |row| {
                row.get::<_, i64>(0)
            })
            .optional()
            .map_err(|e| {
                AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to find next draft in chain")
                    .with_details(e.to_string())
            })?;

        if let Some(id) = next_id {
            current_id = id;
        } else {
            break;
        }
    }

    // Get siblings (other children of parent with different branch)
    let siblings = if let Some(parent_id) = current.parent_draft_id {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, quarter_label, section_type, draft_text, citation_chunk_ids_json,
                       model_name, model_params_hash, prompt_template_version, created_at, artifact_hash,
                       parent_draft_id, revision_number, revision_notes, branch_label
                FROM ai_drafts
                WHERE parent_draft_id = ?1 AND id != ?2
                "#,
            )
            .map_err(|e| {
                AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to query siblings")
                    .with_details(e.to_string())
            })?;

        let mut siblings = Vec::new();
        let rows = stmt.query_map([parent_id, draft_id], |row| {
            let section_type_raw: String = row.get(2)?;
            let section_type = AiDraftSectionType::from_str(&section_type_raw).ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid section_type",
                    )),
                )
            })?;
            let citation_chunk_ids_json: String = row.get(4)?;
            let citation_chunk_ids: Vec<String> =
                serde_json::from_str(&citation_chunk_ids_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;
            Ok(AiDraftArtifact {
                id: row.get(0)?,
                quarter_label: row.get(1)?,
                section_type,
                draft_text: row.get(3)?,
                citation_chunk_ids,
                model_name: row.get(5)?,
                model_params_hash: row.get(6)?,
                prompt_template_version: row.get(7)?,
                created_at: row.get(8)?,
                artifact_hash: row.get(9)?,
                parent_draft_id: row.get(10)?,
                revision_number: row.get(11)?,
                revision_notes: row.get(12)?,
                branch_label: row.get(13)?,
            })
        }).map_err(|e| {
            AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to map sibling rows")
                .with_details(e.to_string())
        })?;

        for r in rows {
            siblings.push(r.map_err(|e| {
                AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to read sibling row")
                    .with_details(e.to_string())
            })?);
        }
        siblings
    } else {
        Vec::new()
    };

    // Get children (direct children of current draft)
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, quarter_label, section_type, draft_text, citation_chunk_ids_json,
                   model_name, model_params_hash, prompt_template_version, created_at, artifact_hash,
                   parent_draft_id, revision_number, revision_notes, branch_label
            FROM ai_drafts
            WHERE parent_draft_id = ?1
            ORDER BY revision_number ASC, id ASC
            "#,
        )
        .map_err(|e| {
            AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to query children")
                .with_details(e.to_string())
        })?;

    let mut children = Vec::new();
    let rows = stmt.query_map([draft_id], |row| {
        let section_type_raw: String = row.get(2)?;
        let section_type = AiDraftSectionType::from_str(&section_type_raw).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid section_type",
                )),
            )
        })?;
        let citation_chunk_ids_json: String = row.get(4)?;
        let citation_chunk_ids: Vec<String> =
            serde_json::from_str(&citation_chunk_ids_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
        Ok(AiDraftArtifact {
            id: row.get(0)?,
            quarter_label: row.get(1)?,
            section_type,
            draft_text: row.get(3)?,
            citation_chunk_ids,
            model_name: row.get(5)?,
            model_params_hash: row.get(6)?,
            prompt_template_version: row.get(7)?,
            created_at: row.get(8)?,
            artifact_hash: row.get(9)?,
            parent_draft_id: row.get(10)?,
            revision_number: row.get(11)?,
            revision_notes: row.get(12)?,
            branch_label: row.get(13)?,
        })
    }).map_err(|e| {
        AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to map child rows")
            .with_details(e.to_string())
    })?;

    for r in rows {
        children.push(r.map_err(|e| {
            AppError::new("DB_AI_DRAFT_QUERY_FAILED", "Failed to read child row")
                .with_details(e.to_string())
        })?);
    }

    Ok(DraftLineage {
        root_id,
        path,
        siblings,
        children,
    })
}
