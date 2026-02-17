[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / SlackIngestSummarySchema

# Variable: SlackIngestSummarySchema

> `const` **SlackIngestSummarySchema**: `ZodObject`\<\{ `detected_format`: `ZodString`; `incident_created`: `ZodBoolean`; `incident_id`: `ZodNumber`; `inserted_events`: `ZodNumber`; `warnings`: `ZodArray`\<`ZodObject`\<\{ `code`: `ZodString`; `details`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; `message`: `ZodString`; \}, `$strip`\>\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:417](https://github.com/saagar210/IncidentReview/blob/17225dffced423acb649d740c08dbd0ee44b59c8/src/lib/schemas.ts#L417)
