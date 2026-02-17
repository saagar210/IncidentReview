// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { SlackImportSection } from "./SlackImportSection";

function baseProps() {
  return {
    incidentOptions: [{ id: 99, external_id: "INC-99", title: "Auth outage" }],
    slackTargetMode: "existing" as const,
    setSlackTargetMode: vi.fn(),
    slackExistingIncidentId: null,
    setSlackExistingIncidentId: vi.fn(),
    slackNewIncidentTitle: "",
    setSlackNewIncidentTitle: vi.fn(),
    slackFileName: "",
    slackText: "",
    setSlackText: vi.fn(),
    setSlackPreview: vi.fn(),
    setSlackSummary: vi.fn(),
    slackPreview: null,
    slackSummary: null,
    onRefreshIncidentsList: vi.fn(),
    onSlackPickFile: vi.fn(),
    onSlackPreview: vi.fn(),
    onSlackIngest: vi.fn(),
  };
}

describe("SlackImportSection", () => {
  it("supports mode switching and clears preview/summary on transcript edits", () => {
    const props = baseProps();

    const { rerender } = render(<SlackImportSection {...props} />);

    expect(screen.getByRole("heading", { name: "Slack Import (Transcript)" })).toBeInTheDocument();
    expect(screen.getByText("Preview shows detected format and warnings (no timestamp guessing)."))
      .toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Refresh incidents" }));
    fireEvent.click(screen.getByRole("button", { name: "Preview" }));
    fireEvent.click(screen.getByRole("button", { name: "Ingest" }));
    expect(props.onRefreshIncidentsList).toHaveBeenCalledTimes(1);
    expect(props.onSlackPreview).toHaveBeenCalledTimes(1);
    expect(props.onSlackIngest).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByLabelText("New Slack-only incident shell"));
    expect(props.setSlackTargetMode).toHaveBeenCalledWith("new");

    fireEvent.change(screen.getByPlaceholderText("Paste transcript text here, or choose a file above."), {
      target: { value: "[2026-02-17 10:00] incident started" },
    });
    expect(props.setSlackText).toHaveBeenCalledWith("[2026-02-17 10:00] incident started");
    expect(props.setSlackPreview).toHaveBeenCalledWith(null);
    expect(props.setSlackSummary).toHaveBeenCalledWith(null);

    rerender(<SlackImportSection {...props} slackTargetMode="new" />);
    expect(screen.getByPlaceholderText("New incident title")).toBeInTheDocument();
  });
});
