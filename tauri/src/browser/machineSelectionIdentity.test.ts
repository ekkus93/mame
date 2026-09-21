import { describe, expect, it } from "vitest";

type Identity = {
  shortName: string | null;
  detailGeneration: number;
  activationGeneration: number;
};

function select(state: Identity, shortName: string | null): Identity {
  if (state.shortName === shortName) return state;
  return {
    shortName,
    detailGeneration: state.detailGeneration + 1,
    activationGeneration: state.activationGeneration + 1,
  };
}

function detailMayCommit(state: Identity, generation: number, shortName: string): boolean {
  return state.detailGeneration === generation && state.shortName === shortName;
}

function activationMayCommit(
  state: Identity,
  detailGeneration: number,
  activationGeneration: number,
  shortName: string,
): boolean {
  return (
    detailMayCommit(state, detailGeneration, shortName) &&
    state.activationGeneration === activationGeneration
  );
}

describe("machine selection async identity", () => {
  it("invalidates a pending detail request when selection clears", () => {
    let state: Identity = { shortName: null, detailGeneration: 0, activationGeneration: 0 };
    state = select(state, "pacman");
    const requestGeneration = state.detailGeneration;
    state = select(state, null);
    expect(detailMayCommit(state, requestGeneration, "pacman")).toBe(false);
  });

  it("rejects out-of-order A detail after B becomes authoritative", () => {
    let state: Identity = { shortName: null, detailGeneration: 0, activationGeneration: 0 };
    state = select(state, "pacman");
    const a = state.detailGeneration;
    state = select(state, "galaga");
    const b = state.detailGeneration;
    expect(detailMayCommit(state, b, "galaga")).toBe(true);
    expect(detailMayCommit(state, a, "pacman")).toBe(false);
  });

  it("invalidates a queued activation when selection identity changes", () => {
    let state: Identity = { shortName: null, detailGeneration: 0, activationGeneration: 0 };
    state = select(state, "pacman");
    const detailGeneration = state.detailGeneration;
    const activationGeneration = state.activationGeneration;
    state = select(state, "galaga");
    expect(
      activationMayCommit(state, detailGeneration, activationGeneration, "pacman"),
    ).toBe(false);
  });
});
