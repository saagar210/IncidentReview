// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { IncidentDetailDrawer } from "./IncidentDetailDrawer";

describe("IncidentDetailDrawer", () => {
  it("renders nothing when closed", () => {
    const { container } = render(
      <IncidentDetailDrawer open={false} loading={false} detail={null} onClose={() => {}} />
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("renders detail state and supports close interactions", () => {
    const onClose = vi.fn();
    render(
      <IncidentDetailDrawer
        open={true}
        loading={false}
        onClose={onClose}
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
            mttd_seconds: 120,
            it_awareness_lag_seconds: null,
            mtta_seconds: 300,
            time_to_mitigation_seconds: null,
            mttr_seconds: 900,
          },
          warnings: [],
          artifacts: [{ id: 10, incident_id: 1, kind: "log", sha256: "abc123def456", filename: "events.log", mime_type: "text/plain", text: null, created_at: "2026-02-17T00:00:00Z" }],
          timeline_events: [
            {
              id: 77,
              incident_id: 1,
              source: "slack",
              ts: "2026-02-17T00:01:00Z",
              author: "bot",
              kind: "message",
              text: "[REDACTED]",
              raw_json: "{\"text_redacted\":true}",
              created_at: "2026-02-17T00:01:00Z",
            },
          ],
        }}
      />
    );

    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.getByText("Computed metrics (deterministic)")).toBeInTheDocument();
    expect(screen.getByText("No warnings.")).toBeInTheDocument();
    expect(screen.getByText("(redacted)")).toBeInTheDocument();
    expect(screen.getByText(/events.log/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    fireEvent.click(screen.getByRole("button", { name: "Close incident detail drawer" }));
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it("renders loading state when detail is pending", () => {
    render(<IncidentDetailDrawer open={true} loading={true} detail={null} onClose={() => {}} />);

    expect(screen.getByText("Loading incident detail...")).toBeInTheDocument();
    expect(screen.getByRole("dialog")).toBeInTheDocument();
  });
});
