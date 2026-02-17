[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / WorkspaceInfoSchema

# Variable: WorkspaceInfoSchema

> `const` **WorkspaceInfoSchema**: `ZodObject`\<\{ `current_db_path`: `ZodString`; `load_error`: `ZodOptional`\<`ZodNullable`\<`ZodObject`\<\{ `code`: `ZodString`; `details`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; `message`: `ZodString`; `retryable`: `ZodBoolean`; \}, `$strip`\>\>\>; `recent_db_paths`: `ZodArray`\<`ZodString`\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:404](https://github.com/saagar210/IncidentReview/blob/17225dffced423acb649d740c08dbd0ee44b59c8/src/lib/schemas.ts#L404)
