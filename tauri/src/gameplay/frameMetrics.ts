import type { FrameMetricsSnapshot } from "../backend/types";

/** Both ages come from Rust's wall-clock receive/ack times, not MAME emulated time. */
export function formatFrameAges(
  metrics: Pick<FrameMetricsSnapshot, "latestReceivedAgeMs" | "lastPresentedAgeMs">,
): string {
  const received = metrics.latestReceivedAgeMs;
  const presented = metrics.lastPresentedAgeMs;
  return `last recv age ${received === null ? "—" : `${received}ms`} · last present age ${presented === null ? "—" : `${presented}ms`}`;
}
