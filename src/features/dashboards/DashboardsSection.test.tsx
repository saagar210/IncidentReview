import { describe, expect, it, vi } from "vitest";
import { renderToString } from "react-dom/server";

vi.mock("echarts-for-react", () => ({ default: () => null }));

import { DashboardsSection } from "./DashboardsSection";

describe("DashboardsSection", () => {
  it("renders empty state when dashboard is not loaded", () => {
    const html = renderToString(
      <DashboardsSection
        dashboard={null}
        selectedSeverity={null}
        setSelectedSeverity={() => {}}
        incidentFilterIds={null}
        incidentFilterLabel=""
        setIncidentFilterIds={() => {}}
        setIncidentFilterLabel={() => {}}
        onOpenIncidentDetail={() => {}}
      />
    );
    expect(html).toContain("Quarter At A Glance");
    expect(html).toContain("Load the dashboard");
  });
});

