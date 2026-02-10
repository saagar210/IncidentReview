import { invoke } from "@tauri-apps/api/core";
import type { ZodType } from "zod";

export type AppError = {
  code: string;
  message: string;
  details?: unknown;
  retryable?: boolean;
};

export class AppErrorException extends Error {
  code: string;
  details?: unknown;
  retryable: boolean;

  constructor(err: AppError) {
    super(`${err.code}: ${err.message}`);
    this.name = "AppErrorException";
    this.code = err.code;
    this.details = err.details;
    this.retryable = err.retryable ?? false;
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object";
}

export function isAppError(value: unknown): value is AppError {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  return (
    typeof v.code === "string" &&
    typeof v.message === "string" &&
    (v.retryable === undefined || typeof v.retryable === "boolean")
  );
}

export function extractAppError(err: unknown): AppError | null {
  if (err instanceof AppErrorException) {
    return { code: err.code, message: err.message.replace(/^[^:]+:\\s*/, ""), details: err.details, retryable: err.retryable };
  }

  // Tauri may wrap error objects; try common shapes without guessing too hard.
  if (isAppError(err)) return err;

  if (isRecord(err) && "error" in err) {
    const inner = (err as Record<string, unknown>).error;
    if (isAppError(inner)) return inner;
  }

  if (typeof err === "string") {
    try {
      const parsed = JSON.parse(err) as unknown;
      if (isAppError(parsed)) return parsed;
      if (isRecord(parsed) && "error" in parsed && isAppError((parsed as Record<string, unknown>).error)) {
        return (parsed as Record<string, unknown>).error as AppError;
      }
    } catch {
      // ignore
    }
  }

  return null;
}

function formatDetails(details: unknown): string {
  if (details == null) return "";
  if (typeof details === "string") return details;
  try {
    return JSON.stringify(details, null, 2);
  } catch {
    return String(details);
  }
}

function formatInvokeError(err: unknown): string {
  const appErr = extractAppError(err);
  if (appErr) {
    const details = appErr.details != null ? `\n\nDetails:\n${formatDetails(appErr.details)}` : "";
    return `${appErr.code}: ${appErr.message}${details}`;
  }
  if (typeof err === "string") return err;
  try {
    return JSON.stringify(err);
  } catch {
    return String(err);
  }
}

export async function invokeValidated<T>(
  command: string,
  args: Record<string, unknown> | undefined,
  schema: ZodType<T> | null
): Promise<T> {
  try {
    const res = await invoke(command, args);
    if (!schema) return res as T;
    return schema.parse(res);
  } catch (e) {
    const appErr = extractAppError(e);
    if (appErr) throw new AppErrorException(appErr);
    throw new Error(formatInvokeError(e));
  }
}
