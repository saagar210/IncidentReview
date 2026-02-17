// @vitest-environment jsdom
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { ReportSection } from "./ReportSection";

describe("ReportSection", () => {
  it("renders read-only markdown output and fallback placeholder", () => {
    const { rerender } = render(<ReportSection reportMd="" />);

    const emptyTextarea = screen.getByPlaceholderText("Generate the report to view Markdown output.");
    expect(emptyTextarea).toHaveAttribute("readonly");

    rerender(<ReportSection reportMd="# Quarterly Incident Review\n\n- Summary" />);
    const renderedTextarea = screen.getByPlaceholderText("Generate the report to view Markdown output.");
    expect((renderedTextarea as HTMLTextAreaElement).value).toContain("Quarterly Incident Review");
    expect((renderedTextarea as HTMLTextAreaElement).value).toContain("- Summary");
  });
});
