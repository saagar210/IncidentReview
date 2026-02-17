import { describe, expect, it } from "vitest";

import { DashboardPayloadV2Schema } from "../../src/lib/schemas";

function buildValidPayload() {
  return {
    version: 2,
    incident_count: 1,
    severity_counts: [{ severity: "SEV-2", count: 1, incident_ids: [42] }],
    incidents: [
      {
        id: 42,
        external_id: "INC-42",
        title: "Database saturation",
        severity: "SEV-2",
        detection_source: "monitoring",
        vendor: "AcmeDB",
        service: "Primary SQL",
        it_awareness_lag_seconds: 120,
        time_to_mitigation_seconds: 600,
        mttr_seconds: 1800,
        warning_count: 0,
      },
    ],
    detection_story: {
      detection_source_mix: [{ key: "monitoring", label: "Monitoring", count: 1, incident_ids: [42] }],
      it_awareness_lag_buckets: [{ key: "0-5m", label: "0-5m", count: 1, incident_ids: [42] }],
    },
    vendor_service_story: {
      top_vendors_by_count: [{ key: "acmedb", label: "AcmeDB", count: 1, incident_ids: [42] }],
      top_services_by_count: [{ key: "primary-sql", label: "Primary SQL", count: 1, incident_ids: [42] }],
      top_vendors_by_pain: [{ key: "acmedb", label: "AcmeDB", count: 1, pain_sum: 55, pain_known_count: 1, incident_ids: [42] }],
      top_services_by_pain: [{ key: "primary-sql", label: "Primary SQL", count: 1, pain_sum: 55, pain_known_count: 1, incident_ids: [42] }],
    },
    response_story: {
      time_to_mitigation_buckets: [{ key: "10-30m", label: "10-30m", count: 1, incident_ids: [42] }],
      time_to_resolve_buckets: [{ key: "30-60m", label: "30-60m", count: 1, incident_ids: [42] }],
    },
  };
}

describe("DashboardPayloadV2 contract", () => {
  it("accepts a valid payload", () => {
    const payload = buildValidPayload();
    const parsed = DashboardPayloadV2Schema.parse(payload);

    expect(parsed.incident_count).toBe(1);
    expect(parsed.response_story.time_to_mitigation_buckets[0].key).toBe("10-30m");
  });

  it("rejects negative counts", () => {
    const payload = buildValidPayload();
    payload.incident_count = -1;

    const result = DashboardPayloadV2Schema.safeParse(payload);

    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues.some((issue) => issue.path.join(".") === "incident_count")).toBe(true);
    }
  });

  it("rejects missing nested story sections", () => {
    const payload = buildValidPayload() as Record<string, unknown>;
    delete payload.response_story;

    const result = DashboardPayloadV2Schema.safeParse(payload);

    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues.some((issue) => issue.path.join(".") === "response_story")).toBe(true);
    }
  });
});
