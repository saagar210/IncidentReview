[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / IncidentSchema

# Variable: IncidentSchema

> `const` **IncidentSchema**: `ZodObject`\<\{ `ack_ts`: `ZodNullable`\<`ZodString`\>; `ack_ts_raw`: `ZodNullable`\<`ZodString`\>; `description`: `ZodNullable`\<`ZodString`\>; `detection_source`: `ZodNullable`\<`ZodString`\>; `external_id`: `ZodNullable`\<`ZodString`\>; `fingerprint`: `ZodString`; `first_observed_ts`: `ZodNullable`\<`ZodString`\>; `first_observed_ts_raw`: `ZodNullable`\<`ZodString`\>; `id`: `ZodNumber`; `impact_pct`: `ZodNullable`\<`ZodNumber`\>; `it_awareness_ts`: `ZodNullable`\<`ZodString`\>; `it_awareness_ts_raw`: `ZodNullable`\<`ZodString`\>; `mitigate_ts`: `ZodNullable`\<`ZodString`\>; `mitigate_ts_raw`: `ZodNullable`\<`ZodString`\>; `resolve_ts`: `ZodNullable`\<`ZodString`\>; `resolve_ts_raw`: `ZodNullable`\<`ZodString`\>; `service`: `ZodNullable`\<`ZodString`\>; `service_health_pct`: `ZodNullable`\<`ZodNumber`\>; `severity`: `ZodNullable`\<`ZodString`\>; `start_ts`: `ZodNullable`\<`ZodString`\>; `start_ts_raw`: `ZodNullable`\<`ZodString`\>; `title`: `ZodString`; `vendor`: `ZodNullable`\<`ZodString`\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:434](https://github.com/saagar210/IncidentReview/blob/fa4457f78085812c15cd94931e9603044d270a42/src/lib/schemas.ts#L434)
