// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { JiraImportSection } from "./JiraImportSection";

function makeProps() {
  return {
    jiraProfiles: [],
    selectedProfileId: null,
    setSelectedProfileId: vi.fn(),
    profileName: "",
    setProfileName: vi.fn(),
    csvFileName: "",
    csvPreview: null,
    mapping: {
      external_id: null,
      title: "",
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
    },
    setMapping: vi.fn(),
    importSummary: null,
    onRefreshProfiles: vi.fn(),
    onPickCsvFile: vi.fn(),
    applyCommonJiraDefaults: vi.fn(),
    onImportCsv: vi.fn(),
    onSaveProfile: vi.fn(),
    onDeleteProfile: vi.fn(),
  };
}

describe("JiraImportSection", () => {
  it("guards mapping defaults by CSV preview and invokes import actions", () => {
    const props = makeProps();
    const { rerender } = render(<JiraImportSection {...props} />);

    expect(screen.getByText("Choose a CSV to preview headers and sample rows.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Apply common Jira defaults" })).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: "Refresh profiles" }));
    fireEvent.click(screen.getByRole("button", { name: "Import CSV" }));
    expect(props.onRefreshProfiles).toHaveBeenCalledTimes(1);
    expect(props.onImportCsv).toHaveBeenCalledTimes(1);

    rerender(
      <JiraImportSection
        {...props}
        csvPreview={{ headers: ["Issue key", "Summary"], rows: [["INC-1", "Outage"]] }}
      />
    );

    expect(screen.getByRole("button", { name: "Apply common Jira defaults" })).toBeEnabled();
    fireEvent.click(screen.getByRole("button", { name: "Apply common Jira defaults" }));
    expect(props.applyCommonJiraDefaults).toHaveBeenCalledTimes(1);
  });
});
