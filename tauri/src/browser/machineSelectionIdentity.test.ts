import { describe, expect, it } from "vitest";

import {
  activationMayCommit,
  detailMayCommit,
  selectMachineIdentity,
} from "./machineAsyncIdentity";

type Identity = Parameters<typeof selectMachineIdentity>[0];

function initialIdentity(): Identity {
  return {
    shortName: null,
    detailGeneration: 0,
    activationGeneration: 0,
  };
}

describe("machine selection async identity", () => {
  it("invalidates a pending detail request when selection clears", () => {
    let state = initialIdentity();
    state = selectMachineIdentity(state, "pacman");
    const requestGeneration = state.detailGeneration;
    state = selectMachineIdentity(state, null);
    expect(detailMayCommit(state, requestGeneration, "pacman")).toBe(false);
  });

  it("rejects out-of-order A detail after B becomes authoritative", () => {
    let state = initialIdentity();
    state = selectMachineIdentity(state, "pacman");
    const a = state.detailGeneration;
    state = selectMachineIdentity(state, "galaga");
    const b = state.detailGeneration;
    expect(detailMayCommit(state, b, "galaga")).toBe(true);
    expect(detailMayCommit(state, a, "pacman")).toBe(false);
  });

  it("invalidates a queued activation when selection identity changes", () => {
    let state = initialIdentity();
    state = selectMachineIdentity(state, "pacman");
    const detailGeneration = state.detailGeneration;
    const activationGeneration = state.activationGeneration;
    state = selectMachineIdentity(state, "galaga");
    expect(activationMayCommit(state, detailGeneration, activationGeneration, "pacman")).toBe(
      false,
    );
  });

  it("allows rapid double-click activation to wait for current machine detail", () => {
    let state = initialIdentity();
    state = selectMachineIdentity(state, "pacman");
    const detailGeneration = state.detailGeneration;
    const doubleClickActivation = state.activationGeneration + 1;
    state = { ...state, activationGeneration: doubleClickActivation };

    expect(
      activationMayCommit(state, detailGeneration, doubleClickActivation, "pacman"),
    ).toBe(true);
  });

  it("allows immediate Enter activation after keyboard selection to wait for selected detail", () => {
    let state = initialIdentity();
    state = selectMachineIdentity(state, "galaga");
    const detailGeneration = state.detailGeneration;
    const enterActivation = state.activationGeneration + 1;
    state = { ...state, activationGeneration: enterActivation };

    expect(activationMayCommit(state, detailGeneration, enterActivation, "galaga")).toBe(true);
  });

  it("rejects rapid A activation when B is selected before A detail returns", () => {
    let state = initialIdentity();
    state = selectMachineIdentity(state, "pacman");
    const aDetailGeneration = state.detailGeneration;
    const aActivationGeneration = state.activationGeneration + 1;
    state = { ...state, activationGeneration: aActivationGeneration };

    state = selectMachineIdentity(state, "galaga");

    expect(
      activationMayCommit(state, aDetailGeneration, aActivationGeneration, "pacman"),
    ).toBe(false);
  });
});
