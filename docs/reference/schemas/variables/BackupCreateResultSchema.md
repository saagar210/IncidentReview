[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / BackupCreateResultSchema

# Variable: BackupCreateResultSchema

> `const` **BackupCreateResultSchema**: `ZodObject`\<\{ `backup_dir`: `ZodString`; `manifest`: `ZodObject`\<\{ `app_version`: `ZodString`; `artifacts`: `ZodObject`\<\{ `files`: `ZodArray`\<`ZodObject`\<\{ `bytes`: `ZodNumber`; `rel_path`: `ZodString`; `sha256`: `ZodString`; \}, `$strip`\>\>; `included`: `ZodBoolean`; \}, `$strip`\>; `counts`: `ZodObject`\<\{ `artifacts_rows`: `ZodNumber`; `incidents`: `ZodNumber`; `timeline_events`: `ZodNumber`; \}, `$strip`\>; `db`: `ZodObject`\<\{ `bytes`: `ZodNumber`; `filename`: `ZodString`; `sha256`: `ZodString`; \}, `$strip`\>; `export_time`: `ZodString`; `manifest_version`: `ZodNumber`; `schema_migrations`: `ZodArray`\<`ZodString`\>; \}, `$strip`\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:363](https://github.com/saagar210/IncidentReview/blob/17225dffced423acb649d740c08dbd0ee44b59c8/src/lib/schemas.ts#L363)
