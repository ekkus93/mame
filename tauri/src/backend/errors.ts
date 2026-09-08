import type { AppErrorEnvelope } from "./types";

export function isAppErrorEnvelope(value: unknown): value is AppErrorEnvelope {
  if (typeof value !== "object" || value === null) {
    return false;
  }

  const candidate = value as Partial<AppErrorEnvelope>;
  return (
    typeof candidate.code === "string" &&
    typeof candidate.message === "string" &&
    typeof candidate.retryable === "boolean" &&
    "details" in candidate
  );
}

export function errorMessage(value: unknown): string {
  if (isAppErrorEnvelope(value)) {
    return value.message;
  }
  if (value instanceof Error) {
    return value.message;
  }
  return "An unknown backend error occurred.";
}
