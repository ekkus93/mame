import { describe, expect, it } from "vitest";

import {
  activationMayCommit,
  detailMayCommit,
  selectMachineIdentity,
} from "./machineAsyncIdentity";

type Identity = Parameters<typeof selectMachineIdentity>[0];

describe("machine selection async identity", () => {
  it("invalidates a pending detail request when selection clears", () => {
    let state: Identity = {
      shortName: null,
      detailGeneration: 0,
      activationGeneration: 0,
    };
    state = selectMachineIdentity(state, "pacman");
    const requestGeneration = state.detailGeneration;
    state = selectMachineIdentity(state, null);
    expect(detailMayCommit(state, requestGeneration, "pacman")).toBe(false);
  });

  it("rejects out-of-order A detail after B becomes authoritative", () => {
    let state: Identity = {
      shortName: null,
      detailGeneration: 0,
      activationGeneration: 0,
    };
    state = selectMachineIdentity(state, "pacman");
    const a = state.detailGeneration;
    state = selectMachineIdentity(state, "galaga");
    const b = state.detailGeneration;
    expect(detailMayCommit(state, b, "galaga")).toBe(true);
    expect(detailMayCommit(state, a, "pacman")).toBe(false);
  });

  it("invalidates a queued activation when selection identity changes", () => {
    let state: Identity = {
      shortName: null,
      detailGeneration: 0,
      activationGeneration: 0,
    };
    state = selectMachineIdentity(state, "pacman");
    const detailGeneration = state.detailGeneration;
    const activationGeneration = state.activationGeneration;
    state = selectMachineIdentity(state, "galaga");
    expect(activationMayCommit(state, detailGeneration, activationGeneration, "pacman")).toBe(
      false,
    );
  });
});
