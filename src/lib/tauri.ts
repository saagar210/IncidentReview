import { invoke } from "@tauri-apps/api/core";
import type { ZodType } from "zod";

type AppError = {
  code: string;
  message: string;
  details?: string | null;
  retryable: boolean;
};

function isAppError(value: unknown): value is AppError {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  return (
    typeof v.code === "string" &&
    typeof v.message === "string" &&
    typeof v.retryable === "boolean" &&
    (v.details === undefined || v.details === null || typeof v.details === "string")
  );
}

function formatInvokeError(err: unknown): string {
  if (isAppError(err)) {
    const details = err.details ? `\n\nDetails:\n${err.details}` : "";
    return `${err.code}: ${err.message}${details}`;
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
    throw new Error(formatInvokeError(e));
  }
}

