import type { MachineListItem } from "../backend/types";

type ScrollableMachineRow = Pick<HTMLElement, "scrollIntoView">;

export function scrollSelectedMachineIntoView(
  items: MachineListItem[],
  selected: MachineListItem | null,
  rows: Array<ScrollableMachineRow | null>,
): boolean {
  if (!selected) return false;
  const index = items.findIndex((machine) => machine.shortName === selected.shortName);
  const row = index >= 0 ? rows[index] : null;
  if (!row) return false;
  row.scrollIntoView({ block: "nearest" });
  return true;
}
