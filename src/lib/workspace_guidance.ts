export function guidanceForWorkspaceErrorCode(code: string): string | null {
  switch (code) {
    case "WORKSPACE_INVALID_PATH":
      return "Pick a valid workspace path. For create: choose an existing folder. For open: choose an existing SQLite DB file.";
    case "WORKSPACE_DB_NOT_FOUND":
      return "Workspace DB file not found. Create a new workspace or pick an existing DB file.";
    case "WORKSPACE_OPEN_FAILED":
      return "Failed to open the workspace DB. If it is in use, close other apps using it and retry.";
    case "WORKSPACE_CREATE_FAILED":
      return "Failed to create the workspace DB. Choose a writable folder and a non-existing filename.";
    case "WORKSPACE_MIGRATION_FAILED":
      return "Failed to migrate the workspace DB schema. If this DB was created by an incompatible version, create a fresh workspace.";
    case "WORKSPACE_PERSIST_FAILED":
      return "Failed to persist workspace selection locally. Your data is not uploaded anywhere, but the app may not remember the last workspace.";
    case "WORKSPACE_DB_LOCKED":
      return "Workspace DB appears locked. Close other processes using the DB and retry.";
    case "WORKSPACE_UNSUPPORTED_SCHEMA_VERSION":
      return "Workspace DB schema is unsupported. Create a fresh workspace or migrate using a compatible app version.";
    default:
      return null;
  }
}

