import type { MachineListItem } from "../backend/types";

type ScrollIntoViewOptions = { block: "nearest" };
type ScrollableMachineRow = Pick<HTMLElement, "scrollIntoView"> &
  Partial<Pick<HTMLElement, "getBoundingClientRect">> & {
    ownerDocument?: Document | null;
  };

export type RowVisibilityGeometry = {
  viewportStart: number;
  viewportEnd: number;
  rowStart: number;
  rowEnd: number;
};

export type SelectedRowVisibilityAction = "already-visible" | "scroll-nearest";

const NEAREST_SCROLL: ScrollIntoViewOptions = { block: "nearest" };

export function selectedRowVisibilityAction({
  viewportStart,
  viewportEnd,
  rowStart,
  rowEnd,
}: RowVisibilityGeometry): SelectedRowVisibilityAction {
  return rowStart >= viewportStart && rowEnd <= viewportEnd ? "already-visible" : "scroll-nearest";
}

function viewportGeometryFor(row: ScrollableMachineRow): RowVisibilityGeometry | null {
  if (!row.getBoundingClientRect) return null;
  const defaultView = row.ownerDocument?.defaultView;
  const viewportEnd = defaultView?.innerHeight ?? null;
  if (viewportEnd === null) return null;
  const rect = row.getBoundingClientRect();
  return {
    viewportStart: 0,
    viewportEnd,
    rowStart: rect.top,
    rowEnd: rect.bottom,
  };
}

export function scrollSelectedMachineIntoView(
  items: MachineListItem[],
  selected: MachineListItem | null,
  rows: Array<ScrollableMachineRow | null>,
): boolean {
  if (!selected) return false;
  const index = items.findIndex((machine) => machine.shortName === selected.shortName);
  const row = index >= 0 ? rows[index] : null;
  if (!row) return false;

  const geometry = viewportGeometryFor(row);
  if (geometry && selectedRowVisibilityAction(geometry) === "already-visible") return false;

  row.scrollIntoView(NEAREST_SCROLL);
  return true;
}
