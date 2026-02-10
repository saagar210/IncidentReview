import { describe, expect, it } from "vitest";
import { renderToString } from "react-dom/server";

import { ValidationSection } from "./ValidationSection";

describe("ValidationSection", () => {
  it("renders when no validation report is loaded", () => {
    const html = renderToString(
      <ValidationSection
        validationReport={null}
        dashboardLoaded={false}
        hasIncidentFilter={false}
        onRefreshValidation={() => {}}
        onRefreshIncidents={() => {}}
        onClearIncidentFilter={() => {}}
        onFilterIncidentFromValidation={() => {}}
      />
    );
    expect(html).toContain("Validation / Anomalies");
  });
});

