import { describe, expect, it } from "vitest";
import { renderToString } from "react-dom/server";

import { IncidentDetailDrawer } from "./IncidentDetailDrawer";

describe("IncidentDetailDrawer", () => {
  it("renders nothing when closed", () => {
    const html = renderToString(
      <IncidentDetailDrawer open={false} loading={false} detail={null} onClose={() => {}} />
    );
    expect(html).toBe("");
  });

  it("renders details when open", () => {
    const html = renderToString(
      <IncidentDetailDrawer
        open={true}
        loading={false}
        onClose={() => {}}
        detail={{
          incident: {
            id: 1,
            external_id: "INC_001",
            fingerprint: "fp",
            title: "Incident INC_001",
            description: null,
            severity: null,
            detection_source: null,
            vendor: null,
            service: null,
            impact_pct: null,
            service_health_pct: null,
            start_ts: null,
            first_observed_ts: null,
            it_awareness_ts: null,
            ack_ts: null,
            mitigate_ts: null,
            resolve_ts: null,
            start_ts_raw: null,
            first_observed_ts_raw: null,
            it_awareness_ts_raw: null,
            ack_ts_raw: null,
            mitigate_ts_raw: null,
            resolve_ts_raw: null,
          },
          metrics: {
            mttd_seconds: null,
            it_awareness_lag_seconds: null,
            mtta_seconds: null,
            time_to_mitigation_seconds: null,
            mttr_seconds: null,
          },
          warnings: [],
          artifacts: [],
          timeline_events: [],
        }}
      />
    );
    expect(html).toContain("Incident detail");
    expect(html).toContain("Computed metrics");
  });
});

