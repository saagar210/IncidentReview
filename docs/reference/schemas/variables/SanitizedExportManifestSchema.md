[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / SanitizedExportManifestSchema

# Variable: SanitizedExportManifestSchema

> `const` **SanitizedExportManifestSchema**: `ZodObject`\<\{ `app_version`: `ZodString`; `export_time`: `ZodString`; `files`: `ZodArray`\<`ZodObject`\<\{ `bytes`: `ZodNumber`; `filename`: `ZodString`; `sha256`: `ZodString`; \}, `$strip`\>\>; `incident_count`: `ZodNumber`; `manifest_version`: `ZodNumber`; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:385](https://github.com/saagar210/IncidentReview/blob/fa4457f78085812c15cd94931e9603044d270a42/src/lib/schemas.ts#L385)
