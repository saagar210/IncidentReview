export function guidanceForSanitizedImportErrorCode(code: string): string | null {
  switch (code) {
    case "INGEST_SANITIZED_DB_NOT_EMPTY":
      return "This import refuses to run on a non-empty DB. Restore or seed into a fresh DB first, then retry.";
    case "INGEST_SANITIZED_MANIFEST_VERSION_MISMATCH":
      return "This sanitized dataset export is from an incompatible version. Re-export the sanitized dataset using a compatible app version.";
    case "INGEST_SANITIZED_MANIFEST_HASH_MISMATCH":
    case "INGEST_SANITIZED_MANIFEST_BYTES_MISMATCH":
      return "This sanitized dataset folder failed integrity checks (hash/size mismatch). Re-export and try again; do not edit files in-place.";
    case "INGEST_SANITIZED_METRICS_MISMATCH":
      return "This sanitized dataset failed deterministic metrics verification. Re-export and try again; if it persists, treat the dataset as corrupted.";
    default:
      return null;
  }
}

