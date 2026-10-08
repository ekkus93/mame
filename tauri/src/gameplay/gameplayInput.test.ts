import { describe, expect, it } from "vitest";

import {
  combineInputState,
  diffInputState,
  gamepadInputState,
  normalizedAxis,
  selectStandardGamepad,
  type GamepadSample,
} from "./gameplayInput";

function gamepad(overrides: Partial<GamepadSample> & Pick<GamepadSample, "id">): GamepadSample {
  return {
    id: overrides.id,
    mapping: overrides.mapping ?? "standard",
    connected: overrides.connected ?? true,
    buttons: overrides.buttons ?? Array.from({ length: 10 }, () => ({ pressed: false, value: 0 })),
    axes: overrides.axes ?? [0, 0],
  };
}

describe("gameplay input mapping", () => {
  it("selects only connected W3C-standard gamepads and honors a preferred device", () => {
    const first = gamepad({ id: "pad-a" });
    const preferred = gamepad({ id: "pad-b" });
    const nonstandard = gamepad({ id: "vendor-map", mapping: "" });
    expect(selectStandardGamepad([nonstandard, first, preferred], "pad-b")?.id).toBe("pad-b");
    expect(selectStandardGamepad([nonstandard, first], "missing")?.id).toBe("pad-a");
    expect(selectStandardGamepad([nonstandard])).toBeNull();
  });

  it("applies deadzone, analog scaling, and digital joystick thresholds", () => {
    const buttons = Array.from({ length: 10 }, () => ({ pressed: false, value: 0 }));
    buttons[0] = { pressed: true, value: 1 };
    buttons[9] = { pressed: true, value: 1 };
    const state = gamepadInputState(
      gamepad({
        id: "pad",
        buttons,
        axes: [0.75, -0.8],
      }),
    );
    expect(state.get("P1_BUTTON1")).toBe(32767);
    expect(state.get("START1")).toBe(32767);
    expect(state.get("P1_JOYSTICK_RIGHT")).toBe(32767);
    expect(state.get("P1_JOYSTICK_UP")).toBe(32767);
    expect(state.get("P1_JOYSTICK_LEFT")).toBe(0);
    expect(state.get("P1_JOYSTICK_DOWN")).toBe(0);
    expect(state.get("P1_AD_STICK_X")).toBeGreaterThan(0);
    expect(state.get("P1_AD_STICK_Y")).toBeLessThan(0);
    expect(normalizedAxis(0.05)).toBe(0);
  });

  it("coalesces keyboard and gamepad state by the strongest magnitude", () => {
    const combined = combineInputState(
      new Map([
        ["P1_BUTTON1", 32767],
        ["P1_AD_STICK_X", -20000],
      ]),
      new Map([
        ["P1_BUTTON1", 0],
        ["P1_AD_STICK_X", 25000],
      ]),
    );
    expect(combined.get("P1_BUTTON1")).toBe(32767);
    expect(combined.get("P1_AD_STICK_X")).toBe(25000);
  });

  it("emits balanced zero releases for every previously accepted input", () => {
    const accepted = new Map([
      ["P1_BUTTON1", 32767],
      ["P1_JOYSTICK_LEFT", 32767],
      ["P1_AD_STICK_X", -12000],
    ]);
    expect(diffInputState(new Map(), accepted)).toEqual([
      { token: "P1_BUTTON1", value: 0 },
      { token: "P1_JOYSTICK_LEFT", value: 0 },
      { token: "P1_AD_STICK_X", value: 0 },
    ]);
  });

  it("does not resend unchanged coalesced input", () => {
    const state = new Map([
      ["P1_BUTTON1", 32767],
      ["P1_AD_STICK_X", 5000],
    ]);
    expect(diffInputState(state, new Map(state))).toEqual([]);
  });
});
