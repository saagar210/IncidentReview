import { describe, expect, it } from "vitest";
import { renderToString } from "react-dom/server";

import { AiSection } from "./AiSection";

describe("AiSection", () => {
  it("renders Phase 5 evidence UI shell", () => {
    const html = renderToString(<AiSection onToast={() => {}} />);
    expect(html).toContain("AI (Phase 5)");
    expect(html).toContain("Add Evidence Source");
    expect(html).toContain("Build Chunks");
    expect(html).toContain("Store AI drafts locally");
    expect(html).toContain("Draft Artifacts (History / Provenance)");
    expect(html).toContain("Evidence Viewer");
  });
});
