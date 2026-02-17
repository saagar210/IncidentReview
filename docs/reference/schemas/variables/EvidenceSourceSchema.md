[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / EvidenceSourceSchema

# Variable: EvidenceSourceSchema

> `const` **EvidenceSourceSchema**: `ZodObject`\<\{ `created_at`: `ZodString`; `label`: `ZodString`; `origin`: `ZodObject`\<\{ `kind`: `ZodString`; `path`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; \}, `$strip`\>; `source_id`: `ZodString`; `type`: `ZodEnum`\<\{ `freeform_text`: `"freeform_text"`; `incident_report_md`: `"incident_report_md"`; `sanitized_export`: `"sanitized_export"`; `slack_transcript`: `"slack_transcript"`; \}\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:220](https://github.com/saagar210/IncidentReview/blob/17225dffced423acb649d740c08dbd0ee44b59c8/src/lib/schemas.ts#L220)
