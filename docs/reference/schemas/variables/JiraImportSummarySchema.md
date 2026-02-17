[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / JiraImportSummarySchema

# Variable: JiraImportSummarySchema

> `const` **JiraImportSummarySchema**: `ZodObject`\<\{ `conflicts`: `ZodArray`\<`ZodObject`\<\{ `external_id`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; `fingerprint`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; `reason`: `ZodString`; `row`: `ZodNumber`; \}, `$strip`\>\>; `inserted`: `ZodNumber`; `skipped`: `ZodNumber`; `updated`: `ZodNumber`; `warnings`: `ZodArray`\<`ZodObject`\<\{ `code`: `ZodString`; `details`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; `message`: `ZodString`; \}, `$strip`\>\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:60](https://github.com/saagar210/IncidentReview/blob/fa4457f78085812c15cd94931e9603044d270a42/src/lib/schemas.ts#L60)
