import type { MachineListItem } from "../backend/types";

type ScrollIntoViewOptions = { block: "nearest" };
type ScrollableMachineRow = Pick<HTMLElement, "scrollIntoView"> &
  Partial<Pick<HTMLElement, "getBoundingClientRect">>;
type MachineListViewport = Partial<Pick<HTMLElement, "getBoundingClientRect">>;

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

function visibilityGeometryFor(
  row: ScrollableMachineRow,
  viewport: MachineListViewport | null,
): RowVisibilityGeometry | null {
  if (!row.getBoundingClientRect || !viewport?.getBoundingClientRect) return null;
  const rowRect = row.getBoundingClientRect();
  const viewportRect = viewport.getBoundingClientRect();
  return {
    viewportStart: viewportRect.top,
    viewportEnd: viewportRect.bottom,
    rowStart: rowRect.top,
    rowEnd: rowRect.bottom,
  };
}

export function scrollSelectedMachineIntoView(
  items: MachineListItem[],
  selected: MachineListItem | null,
  rows: Array<ScrollableMachineRow | null>,
  viewport: MachineListViewport | null = null,
): boolean {
  if (!selected) return false;
  const index = items.findIndex((machine) => machine.shortName === selected.shortName);
  const row = index >= 0 ? rows[index] : null;
  if (!row) return false;

  const geometry = visibilityGeometryFor(row, viewport);
  if (geometry && selectedRowVisibilityAction(geometry) === "already-visible") return false;

  row.scrollIntoView(NEAREST_SCROLL);
  return true;
}
