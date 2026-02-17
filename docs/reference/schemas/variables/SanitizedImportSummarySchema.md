[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / SanitizedImportSummarySchema

# Variable: SanitizedImportSummarySchema

> `const` **SanitizedImportSummarySchema**: `ZodObject`\<\{ `import_warnings`: `ZodArray`\<`ZodObject`\<\{ `code`: `ZodString`; `details`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; `message`: `ZodString`; \}, `$strip`\>\>; `inserted_incidents`: `ZodNumber`; `inserted_timeline_events`: `ZodNumber`; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:393](https://github.com/saagar210/IncidentReview/blob/fa4457f78085812c15cd94931e9603044d270a42/src/lib/schemas.ts#L393)
