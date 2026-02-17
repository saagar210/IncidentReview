[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / EvidenceSourceListSchema

# Variable: EvidenceSourceListSchema

> `const` **EvidenceSourceListSchema**: `ZodArray`\<`ZodObject`\<\{ `created_at`: `ZodString`; `label`: `ZodString`; `origin`: `ZodObject`\<\{ `kind`: `ZodString`; `path`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; \}, `$strip`\>; `source_id`: `ZodString`; `type`: `ZodEnum`\<\{ `freeform_text`: `"freeform_text"`; `incident_report_md`: `"incident_report_md"`; `sanitized_export`: `"sanitized_export"`; `slack_transcript`: `"slack_transcript"`; \}\>; \}, `$strip`\>\>

Defined in: [src/lib/schemas.ts:228](https://github.com/saagar210/IncidentReview/blob/fa4457f78085812c15cd94931e9603044d270a42/src/lib/schemas.ts#L228)
