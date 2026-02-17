[**incidentreview**](../../README.md)

***

[incidentreview](../../README.md) / [schemas](../README.md) / VendorServiceStoryV1Schema

# Variable: VendorServiceStoryV1Schema

> `const` **VendorServiceStoryV1Schema**: `ZodObject`\<\{ `top_services_by_count`: `ZodArray`\<`ZodObject`\<\{ `count`: `ZodNumber`; `incident_ids`: `ZodArray`\<`ZodNumber`\>; `key`: `ZodString`; `label`: `ZodString`; \}, `$strip`\>\>; `top_services_by_pain`: `ZodArray`\<`ZodObject`\<\{ `count`: `ZodNumber`; `incident_ids`: `ZodArray`\<`ZodNumber`\>; `key`: `ZodString`; `label`: `ZodString`; `pain_known_count`: `ZodNumber`; `pain_sum`: `ZodNumber`; \}, `$strip`\>\>; `top_vendors_by_count`: `ZodArray`\<`ZodObject`\<\{ `count`: `ZodNumber`; `incident_ids`: `ZodArray`\<`ZodNumber`\>; `key`: `ZodString`; `label`: `ZodString`; \}, `$strip`\>\>; `top_vendors_by_pain`: `ZodArray`\<`ZodObject`\<\{ `count`: `ZodNumber`; `incident_ids`: `ZodArray`\<`ZodNumber`\>; `key`: `ZodString`; `label`: `ZodString`; `pain_known_count`: `ZodNumber`; `pain_sum`: `ZodNumber`; \}, `$strip`\>\>; \}, `$strip`\>

Defined in: [src/lib/schemas.ts:143](https://github.com/saagar210/IncidentReview/blob/fa4457f78085812c15cd94931e9603044d270a42/src/lib/schemas.ts#L143)
