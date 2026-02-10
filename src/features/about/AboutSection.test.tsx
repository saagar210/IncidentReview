import { describe, expect, it } from "vitest";
import { renderToString } from "react-dom/server";

import { AboutSection } from "./AboutSection";

describe("AboutSection", () => {
  it("renders About panel shell", () => {
    const html = renderToString(<AboutSection />);
    expect(html).toContain("About");
    expect(html).toContain("Git commit");
    expect(html).toContain("Schema / Migrations");
    expect(html).toContain("Local AI (Ollama)");
  });
});

