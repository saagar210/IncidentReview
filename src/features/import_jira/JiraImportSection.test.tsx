import { describe, expect, it } from "vitest";
import { renderToString } from "react-dom/server";

import { JiraImportSection } from "./JiraImportSection";

describe("JiraImportSection", () => {
  it("renders with no CSV selected", () => {
    const html = renderToString(
      <JiraImportSection
        jiraProfiles={[]}
        selectedProfileId={null}
        setSelectedProfileId={() => {}}
        profileName=""
        setProfileName={() => {}}
        csvFileName=""
        csvPreview={null}
        mapping={{
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
        }}
        setMapping={() => {}}
        importSummary={null}
        onRefreshProfiles={() => {}}
        onPickCsvFile={() => {}}
        applyCommonJiraDefaults={() => {}}
        onImportCsv={() => {}}
        onSaveProfile={() => {}}
        onDeleteProfile={() => {}}
      />
    );

    expect(html).toContain("Jira CSV Import");
    expect(html).toContain("Choose a CSV to preview headers");
  });
});

