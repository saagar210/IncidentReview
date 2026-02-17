[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / SlackPreviewSchema

# Variable: SlackPreviewSchema

> `const` **SlackPreviewSchema**: `ZodObject`\<\{ `detected_format`: `ZodString`; `line_count`: `ZodNumber`; `message_count`: `ZodNumber`; `warnings`: `ZodArray`\<`ZodObject`\<\{ `code`: `ZodString`; `details`: `ZodOptional`\<`ZodNullable`\<`ZodString`\>\>; `message`: `ZodString`; \}, `$strip`\>\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:410](https://github.com/saagar210/IncidentReview/blob/17225dffced423acb649d740c08dbd0ee44b59c8/src/lib/schemas.ts#L410)
