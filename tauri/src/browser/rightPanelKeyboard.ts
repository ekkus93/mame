export type PrimaryRightView = "images" | "info";

export function nextPrimaryRightView(
  current: PrimaryRightView,
  key: string,
): PrimaryRightView | null {
  if (key === "ArrowRight") return "info";
  if (key === "ArrowLeft") return current === "info" ? "images" : null;
  return current;
}
