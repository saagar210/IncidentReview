import { describe, expect, it } from "vitest";
import { renderToString } from "react-dom/server";

import { SlackImportSection } from "./SlackImportSection";

describe("SlackImportSection", () => {
  it("renders with empty state", () => {
    const html = renderToString(
      <SlackImportSection
        incidentOptions={[]}
        slackTargetMode="existing"
        setSlackTargetMode={() => {}}
        slackExistingIncidentId={null}
        setSlackExistingIncidentId={() => {}}
        slackNewIncidentTitle=""
        setSlackNewIncidentTitle={() => {}}
        slackFileName=""
        slackText=""
        setSlackText={() => {}}
        setSlackPreview={() => {}}
        setSlackSummary={() => {}}
        slackPreview={null}
        slackSummary={null}
        onRefreshIncidentsList={() => {}}
        onSlackPickFile={() => {}}
        onSlackPreview={() => {}}
        onSlackIngest={() => {}}
      />
    );

    expect(html).toContain("Slack Import");
    expect(html).toContain("Transcript");
  });
});

