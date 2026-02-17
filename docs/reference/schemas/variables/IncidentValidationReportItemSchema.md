[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / IncidentValidationReportItemSchema

# Variable: IncidentValidationReportItemSchema

> `const` **IncidentValidationReportItemSchema**: `ZodObject`\<\{ `external_id`: `ZodNullable`\<`ZodString`\>; `id`: `ZodNumber`; `title`: `ZodString`; `warnings`: `ZodArray`\<`ZodObject`\<\{ `code`: `ZodString`; `details`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; `message`: `ZodString`; \}, `$strip`\>\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:425](https://github.com/saagar210/IncidentReview/blob/fa4457f78085812c15cd94931e9603044d270a42/src/lib/schemas.ts#L425)
