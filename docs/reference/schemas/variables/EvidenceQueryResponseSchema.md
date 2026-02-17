[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / EvidenceQueryResponseSchema

# Variable: EvidenceQueryResponseSchema

> `const` **EvidenceQueryResponseSchema**: `ZodObject`\<\{ `hits`: `ZodArray`\<`ZodObject`\<\{ `chunk_id`: `ZodString`; `citation`: `ZodObject`\<\{ `chunk_id`: `ZodString`; `locator`: `ZodObject`\<\{ `char_range`: `ZodOptional`\<`ZodNullable`\<...\>\>; `ordinal`: `ZodNumber`; `source_id`: `ZodString`; `text_sha256`: `ZodString`; \}, `$strip`\>; \}, `$strip`\>; `score`: `ZodNumber`; `snippet`: `ZodString`; `source_id`: `ZodString`; \}, `$strip`\>\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:287](https://github.com/saagar210/IncidentReview/blob/fa4457f78085812c15cd94931e9603044d270a42/src/lib/schemas.ts#L287)
