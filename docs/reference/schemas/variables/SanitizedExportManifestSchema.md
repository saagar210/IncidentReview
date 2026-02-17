[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / SanitizedExportManifestSchema

# Variable: SanitizedExportManifestSchema

> `const` **SanitizedExportManifestSchema**: `ZodObject`\<\{ `app_version`: `ZodString`; `export_time`: `ZodString`; `files`: `ZodArray`\<`ZodObject`\<\{ `bytes`: `ZodNumber`; `filename`: `ZodString`; `sha256`: `ZodString`; \}, `$strip`\>\>; `incident_count`: `ZodNumber`; `manifest_version`: `ZodNumber`; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:385](https://github.com/saagar210/IncidentReview/blob/17225dffced423acb649d740c08dbd0ee44b59c8/src/lib/schemas.ts#L385)
