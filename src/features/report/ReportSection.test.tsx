import { describe, expect, it } from "vitest";
import { renderToString } from "react-dom/server";

import { ReportSection } from "./ReportSection";

describe("ReportSection", () => {
  it("renders a report textarea", () => {
    const html = renderToString(<ReportSection reportMd="" />);
    expect(html).toContain("QIR Report");
  });
});

