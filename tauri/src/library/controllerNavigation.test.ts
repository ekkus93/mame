import { describe, expect, it } from "vitest";

import {
  INITIAL_CONTROLLER_NAVIGATION_STATE,
  type ControllerNavigationState,
  type ControllerSample,
  updateControllerNavigation,
} from "./controllerNavigation";

const NEUTRAL_SAMPLE: ControllerSample = {
  connected: true,
  standardMapping: true,
  dpadUp: false,
  dpadDown: false,
  primaryPressed: false,
  leftStickY: 0,
};

const OWNED_BY_FRONTEND = { documentFocused: true, gameplayActive: false };

function armedState(): ControllerNavigationState {
  return { ...INITIAL_CONTROLLER_NAVIGATION_STATE, armed: true };
}

describe("controller navigation prototype", () => {
  it("fails closed when MAME owns gameplay input or the Tauri document is unfocused", () => {
    const pressed = { ...NEUTRAL_SAMPLE, dpadDown: true };

    expect(
      updateControllerNavigation(armedState(), pressed, {
        documentFocused: true,
        gameplayActive: true,
      }),
    ).toEqual({ state: INITIAL_CONTROLLER_NAVIGATION_STATE, actions: [] });
    expect(
      updateControllerNavigation(armedState(), pressed, {
        documentFocused: false,
        gameplayActive: false,
      }),
    ).toEqual({ state: INITIAL_CONTROLLER_NAVIGATION_STATE, actions: [] });
  });

  it("requires a neutral frame before accepting input after ownership returns", () => {
    const held = { ...NEUTRAL_SAMPLE, dpadDown: true };
    const stillHeld = updateControllerNavigation(
      INITIAL_CONTROLLER_NAVIGATION_STATE,
      held,
      OWNED_BY_FRONTEND,
    );
    expect(stillHeld.actions).toEqual([]);
    expect(stillHeld.state.armed).toBe(false);

    const released = updateControllerNavigation(stillHeld.state, NEUTRAL_SAMPLE, OWNED_BY_FRONTEND);
    expect(released.actions).toEqual([]);
    expect(released.state.armed).toBe(true);

    const pressedAgain = updateControllerNavigation(released.state, held, OWNED_BY_FRONTEND);
    expect(pressedAgain.actions).toEqual(["nextMachine"]);
  });

  it("emits one bounded D-pad movement edge instead of repeating every poll", () => {
    const pressed = { ...NEUTRAL_SAMPLE, dpadUp: true };
    const first = updateControllerNavigation(armedState(), pressed, OWNED_BY_FRONTEND);
    expect(first.actions).toEqual(["previousMachine"]);

    const held = updateControllerNavigation(first.state, pressed, OWNED_BY_FRONTEND);
    expect(held.actions).toEqual([]);
  });

  it("treats a sampled direction reversal as a new movement edge", () => {
    const up = updateControllerNavigation(
      armedState(),
      { ...NEUTRAL_SAMPLE, leftStickY: -0.8 },
      OWNED_BY_FRONTEND,
    );
    expect(up.actions).toEqual(["previousMachine"]);

    const down = updateControllerNavigation(
      up.state,
      { ...NEUTRAL_SAMPLE, leftStickY: 0.8 },
      OWNED_BY_FRONTEND,
    );
    expect(down.actions).toEqual(["nextMachine"]);
  });

  it("uses hysteresis for left-stick vertical navigation", () => {
    const entered = updateControllerNavigation(
      armedState(),
      { ...NEUTRAL_SAMPLE, leftStickY: 0.8 },
      OWNED_BY_FRONTEND,
    );
    expect(entered.actions).toEqual(["nextMachine"]);

    const noisy = updateControllerNavigation(
      entered.state,
      { ...NEUTRAL_SAMPLE, leftStickY: 0.5 },
      OWNED_BY_FRONTEND,
    );
    expect(noisy.actions).toEqual([]);
    expect(noisy.state.verticalDirection).toBe(1);

    const released = updateControllerNavigation(
      noisy.state,
      { ...NEUTRAL_SAMPLE, leftStickY: 0.2 },
      OWNED_BY_FRONTEND,
    );
    expect(released.state.verticalDirection).toBe(0);
  });

  it("emits the south/A button as a single activation edge", () => {
    const pressed = { ...NEUTRAL_SAMPLE, primaryPressed: true };
    const first = updateControllerNavigation(armedState(), pressed, OWNED_BY_FRONTEND);
    expect(first.actions).toEqual(["activateFocused"]);

    const held = updateControllerNavigation(first.state, pressed, OWNED_BY_FRONTEND);
    expect(held.actions).toEqual([]);
  });

  it("rejects disconnected and non-standard gamepads", () => {
    expect(
      updateControllerNavigation(
        armedState(),
        { ...NEUTRAL_SAMPLE, connected: false, dpadDown: true },
        OWNED_BY_FRONTEND,
      ),
    ).toEqual({ state: INITIAL_CONTROLLER_NAVIGATION_STATE, actions: [] });
    expect(
      updateControllerNavigation(
        armedState(),
        { ...NEUTRAL_SAMPLE, standardMapping: false, dpadDown: true },
        OWNED_BY_FRONTEND,
      ),
    ).toEqual({ state: INITIAL_CONTROLLER_NAVIGATION_STATE, actions: [] });
  });
});
