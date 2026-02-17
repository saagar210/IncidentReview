[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / SlackPreviewSchema

# Variable: SlackPreviewSchema

> `const` **SlackPreviewSchema**: `ZodObject`\<\{ `detected_format`: `ZodString`; `line_count`: `ZodNumber`; `message_count`: `ZodNumber`; `warnings`: `ZodArray`\<`ZodObject`\<\{ `code`: `ZodString`; `details`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; `message`: `ZodString`; \}, `$strip`\>\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:410](https://github.com/saagar210/IncidentReview/blob/fa4457f78085812c15cd94931e9603044d270a42/src/lib/schemas.ts#L410)
