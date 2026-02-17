// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ValidationSection } from "./ValidationSection";

describe("ValidationSection", () => {
  it("enforces filter controls and forwards validation drill-down actions", () => {
    const onRefreshValidation = vi.fn();
    const onRefreshIncidents = vi.fn();
    const onClearIncidentFilter = vi.fn();
    const onFilterIncidentFromValidation = vi.fn();

    const { rerender } = render(
      <ValidationSection
        validationReport={null}
        dashboardLoaded={false}
        hasIncidentFilter={false}
        onRefreshValidation={onRefreshValidation}
        onRefreshIncidents={onRefreshIncidents}
        onClearIncidentFilter={onClearIncidentFilter}
        onFilterIncidentFromValidation={onFilterIncidentFromValidation}
      />
    );

    expect(screen.getByText("Load validation to see incidents with warnings/errors.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Clear incident filter" })).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: "Refresh validation" }));
    fireEvent.click(screen.getByRole("button", { name: "Refresh incidents" }));
    expect(onRefreshValidation).toHaveBeenCalledTimes(1);
    expect(onRefreshIncidents).toHaveBeenCalledTimes(1);

    rerender(
      <ValidationSection
        validationReport={[
          {
            id: 12,
            external_id: "INC-12",
            title: "API latency spike",
            warnings: [{ code: "VALIDATION_ORDER", message: "Timestamp ordering violation" }],
          },
        ]}
        dashboardLoaded={true}
        hasIncidentFilter={true}
        onRefreshValidation={onRefreshValidation}
        onRefreshIncidents={onRefreshIncidents}
        onClearIncidentFilter={onClearIncidentFilter}
        onFilterIncidentFromValidation={onFilterIncidentFromValidation}
      />
    );

    expect(screen.getByText(/Timestamp ordering violation/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Filter incidents table" }));
    expect(onFilterIncidentFromValidation).toHaveBeenCalledWith(12, "validation:INC-12");

    fireEvent.click(screen.getByRole("button", { name: "Clear incident filter" }));
    expect(onClearIncidentFilter).toHaveBeenCalledTimes(1);
  });
});
