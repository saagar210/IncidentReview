[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / AiDraftResponseSchema

# Variable: AiDraftResponseSchema

> `const` **AiDraftResponseSchema**: `ZodObject`\<\{ `citations`: `ZodArray`\<`ZodObject`\<\{ `chunk_id`: `ZodString`; `locator`: `ZodObject`\<\{ `char_range`: `ZodOptional`\<`ZodNullable`\<`ZodTuple`\<\[..., ...\], `null`\>\>\>; `ordinal`: `ZodNumber`; `source_id`: `ZodString`; `text_sha256`: `ZodString`; \}, `$strip`\>; \}, `$strip`\>\>; `markdown`: `ZodString`; `model_name`: `ZodString`; `model_params_hash`: `ZodString`; `prompt_template_version`: `ZodString`; `section_id`: `ZodEnum`\<\{ `action_plan_next_quarter`: `"action_plan_next_quarter"`; `exec_summary`: `"exec_summary"`; `incident_highlights_top_n`: `"incident_highlights_top_n"`; `quarter_narrative_recap`: `"quarter_narrative_recap"`; `theme_analysis`: `"theme_analysis"`; \}\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:299](https://github.com/saagar210/IncidentReview/blob/17225dffced423acb649d740c08dbd0ee44b59c8/src/lib/schemas.ts#L299)
