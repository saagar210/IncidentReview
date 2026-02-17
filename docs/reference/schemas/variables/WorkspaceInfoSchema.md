[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / WorkspaceInfoSchema

# Variable: WorkspaceInfoSchema

> `const` **WorkspaceInfoSchema**: `ZodObject`\<\{ `current_db_path`: `ZodString`; `load_error`: `ZodOptional`\<`ZodNullable`\<`ZodObject`\<\{ `code`: `ZodString`; `details`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; `message`: `ZodString`; `retryable`: `ZodBoolean`; \}, `$strip`\>\>\>; `recent_db_paths`: `ZodArray`\<`ZodString`\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:404](https://github.com/saagar210/IncidentReview/blob/fa4457f78085812c15cd94931e9603044d270a42/src/lib/schemas.ts#L404)
