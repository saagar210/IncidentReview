// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("echarts-for-react", () => ({ default: () => null }));

import { DashboardsSection } from "./DashboardsSection";

const dashboard = {
  version: 2,
  incident_count: 1,
  severity_counts: [{ severity: "SEV-2", count: 1, incident_ids: [101] }],
  incidents: [
    {
      id: 101,
      external_id: "INC-101",
      title: "Database saturation",
      severity: "SEV-2",
      detection_source: "monitoring",
      vendor: "AcmeDB",
      service: "Primary SQL",
      it_awareness_lag_seconds: 60,
      time_to_mitigation_seconds: 300,
      mttr_seconds: 1800,
      warning_count: 1,
    },
  ],
  detection_story: {
    detection_source_mix: [{ key: "monitoring", label: "Monitoring", count: 1, incident_ids: [101] }],
    it_awareness_lag_buckets: [{ key: "0-5m", label: "0-5m", count: 1, incident_ids: [101] }],
  },
  vendor_service_story: {
    top_vendors_by_count: [{ key: "acmedb", label: "AcmeDB", count: 1, incident_ids: [101] }],
    top_services_by_count: [{ key: "primary-sql", label: "Primary SQL", count: 1, incident_ids: [101] }],
    top_vendors_by_pain: [{ key: "acmedb", label: "AcmeDB", count: 1, pain_sum: 10, pain_known_count: 1, incident_ids: [101] }],
    top_services_by_pain: [{ key: "primary-sql", label: "Primary SQL", count: 1, pain_sum: 10, pain_known_count: 1, incident_ids: [101] }],
  },
  response_story: {
    time_to_mitigation_buckets: [{ key: "0-10m", label: "0-10m", count: 1, incident_ids: [101] }],
    time_to_resolve_buckets: [{ key: "10-30m", label: "10-30m", count: 1, incident_ids: [101] }],
  },
};

describe("DashboardsSection", () => {
  it("shows empty state and supports drill-down actions when loaded", () => {
    const setSelectedSeverity = vi.fn();
    const setIncidentFilterIds = vi.fn();
    const setIncidentFilterLabel = vi.fn();
    const onOpenIncidentDetail = vi.fn();

    const { rerender } = render(
      <DashboardsSection
        dashboard={null}
        selectedSeverity={null}
        setSelectedSeverity={setSelectedSeverity}
        incidentFilterIds={null}
        incidentFilterLabel=""
        setIncidentFilterIds={setIncidentFilterIds}
        setIncidentFilterLabel={setIncidentFilterLabel}
        onOpenIncidentDetail={onOpenIncidentDetail}
      />
    );

    expect(screen.getByText("Load the dashboard to view severity distribution and incidents.")).toBeInTheDocument();

    rerender(
      <DashboardsSection
        dashboard={dashboard}
        selectedSeverity={null}
        setSelectedSeverity={setSelectedSeverity}
        incidentFilterIds={null}
        incidentFilterLabel=""
        setIncidentFilterIds={setIncidentFilterIds}
        setIncidentFilterLabel={setIncidentFilterLabel}
        onOpenIncidentDetail={onOpenIncidentDetail}
      />
    );

    expect(screen.getByText("Incident Count")).toBeInTheDocument();
    expect(screen.getByText("Database saturation")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Database saturation" }));
    expect(onOpenIncidentDetail).toHaveBeenCalledWith(101);

    expect(screen.getByRole("button", { name: "Clear incident filter" })).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Clear filter" }));
    expect(setSelectedSeverity).toHaveBeenCalledWith(null);
  });
});
