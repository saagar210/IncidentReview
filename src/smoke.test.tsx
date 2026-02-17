// @vitest-environment jsdom
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { AppNav } from "./ui/AppNav";

describe("smoke", () => {
  it("renders navigation shell with deterministic metrics hint", () => {
    render(
      <AppNav
        items={[
          { label: "Workspace", href: "#workspace" },
          { label: "Dashboards", href: "#dashboards", kind: "accent" },
        ]}
      />
    );

    expect(screen.getByRole("navigation", { name: "Navigation" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Dashboards" })).toHaveClass("btn--accent");
    expect(screen.getByText(/All metrics are computed deterministically in Rust/)).toBeInTheDocument();
  });
});
