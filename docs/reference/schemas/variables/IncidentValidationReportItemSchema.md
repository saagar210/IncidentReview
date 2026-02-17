[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / IncidentValidationReportItemSchema

# Variable: IncidentValidationReportItemSchema

> `const` **IncidentValidationReportItemSchema**: `ZodObject`\<\{ `external_id`: `ZodNullable`\<`ZodString`\>; `id`: `ZodNumber`; `title`: `ZodString`; `warnings`: `ZodArray`\<`ZodObject`\<\{ `code`: `ZodString`; `details`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; `message`: `ZodString`; \}, `$strip`\>\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:425](https://github.com/saagar210/IncidentReview/blob/17225dffced423acb649d740c08dbd0ee44b59c8/src/lib/schemas.ts#L425)
