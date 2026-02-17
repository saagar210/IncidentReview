[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / EvidenceContextResponseSchema

# Variable: EvidenceContextResponseSchema

> `const` **EvidenceContextResponseSchema**: `ZodObject`\<\{ `centerChunkId`: `ZodString`; `chunks`: `ZodArray`\<`ZodObject`\<\{ `chunk_id`: `ZodString`; `meta`: `ZodObject`\<\{ `incident_keys`: `ZodOptional`\<`ZodNullable`\<`ZodArray`\<`ZodString`\>\>\>; `kind`: `ZodString`; `time_range`: `ZodOptional`\<`ZodNullable`\<`ZodObject`\<\{ `end_ts`: ...; `start_ts`: ...; \}, `$strip`\>\>\>; \}, `$strip`\>; `ordinal`: `ZodNumber`; `source_id`: `ZodString`; `text_sha256`: `ZodString`; `token_count_est`: `ZodNumber`; \}, `$strip`\>\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:256](https://github.com/saagar210/IncidentReview/blob/fa4457f78085812c15cd94931e9603044d270a42/src/lib/schemas.ts#L256)
