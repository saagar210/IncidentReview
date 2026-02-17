[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / SlackIngestSummarySchema

# Variable: SlackIngestSummarySchema

> `const` **SlackIngestSummarySchema**: `ZodObject`\<\{ `detected_format`: `ZodString`; `incident_created`: `ZodBoolean`; `incident_id`: `ZodNumber`; `inserted_events`: `ZodNumber`; `warnings`: `ZodArray`\<`ZodObject`\<\{ `code`: `ZodString`; `details`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; `message`: `ZodString`; \}, `$strip`\>\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:417](https://github.com/saagar210/IncidentReview/blob/fa4457f78085812c15cd94931e9603044d270a42/src/lib/schemas.ts#L417)
