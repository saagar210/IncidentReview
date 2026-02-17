[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / IncidentSummaryV2Schema

# Variable: IncidentSummaryV2Schema

> `const` **IncidentSummaryV2Schema**: `ZodObject`\<\{ `detection_source`: `ZodNullable`\<`ZodString`\>; `external_id`: `ZodNullable`\<`ZodString`\>; `id`: `ZodNumber`; `it_awareness_lag_seconds`: `ZodNullable`\<`ZodNumber`\>; `mttr_seconds`: `ZodNullable`\<`ZodNumber`\>; `service`: `ZodNullable`\<`ZodString`\>; `severity`: `ZodNullable`\<`ZodString`\>; `time_to_mitigation_seconds`: `ZodNullable`\<`ZodNumber`\>; `title`: `ZodString`; `vendor`: `ZodNullable`\<`ZodString`\>; `warning_count`: `ZodNumber`; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:155](https://github.com/saagar210/IncidentReview/blob/17225dffced423acb649d740c08dbd0ee44b59c8/src/lib/schemas.ts#L155)
