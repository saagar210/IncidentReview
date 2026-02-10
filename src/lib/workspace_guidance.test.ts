import { describe, expect, it } from "vitest";

import { guidanceForWorkspaceErrorCode } from "./workspace_guidance";

describe("guidanceForWorkspaceErrorCode", () => {
  it("returns guidance for required DS6 codes", () => {
    expect(guidanceForWorkspaceErrorCode("WORKSPACE_INVALID_PATH")).toBeTruthy();
    expect(guidanceForWorkspaceErrorCode("WORKSPACE_DB_NOT_FOUND")).toBeTruthy();
    expect(guidanceForWorkspaceErrorCode("WORKSPACE_OPEN_FAILED")).toBeTruthy();
    expect(guidanceForWorkspaceErrorCode("WORKSPACE_CREATE_FAILED")).toBeTruthy();
    expect(guidanceForWorkspaceErrorCode("WORKSPACE_MIGRATION_FAILED")).toBeTruthy();
    expect(guidanceForWorkspaceErrorCode("WORKSPACE_PERSIST_FAILED")).toBeTruthy();
  });
});

