import type { MameInputUpdate } from "../backend/types";

export const KEYBOARD_INPUTS: Readonly<Record<string, string>> = {
  ArrowUp: "P1_JOYSTICK_UP",
  ArrowDown: "P1_JOYSTICK_DOWN",
  ArrowLeft: "P1_JOYSTICK_LEFT",
  ArrowRight: "P1_JOYSTICK_RIGHT",
  KeyZ: "P1_BUTTON1",
  KeyX: "P1_BUTTON2",
  KeyC: "P1_BUTTON3",
  KeyV: "P1_BUTTON4",
  Enter: "START1",
  Digit5: "COIN1",
};

const GAMEPAD_BUTTON_INPUTS: Readonly<Record<number, string>> = {
  0: "P1_BUTTON1",
  1: "P1_BUTTON2",
  2: "P1_BUTTON3",
  3: "P1_BUTTON4",
  8: "COIN1",
  9: "START1",
};

export const GAMEPAD_DEADZONE = 0.12;
export const GAMEPAD_DIGITAL_THRESHOLD = 0.5;

export type GamepadSample = {
  id: string;
  mapping: string;
  connected: boolean;
  buttons: readonly { pressed: boolean; value: number }[];
  axes: readonly number[];
};

export function normalizedAxis(value: number): number {
  if (!Number.isFinite(value) || Math.abs(value) < GAMEPAD_DEADZONE) return 0;
  return Math.max(-32768, Math.min(32767, Math.round(value * 32767)));
}

export function selectStandardGamepad(
  gamepads: readonly (GamepadSample | null)[],
  preferredId?: string | null,
): GamepadSample | null {
  const connected = gamepads.filter(
    (gamepad): gamepad is GamepadSample =>
      gamepad !== null && gamepad.connected && gamepad.mapping === "standard",
  );
  if (preferredId) {
    return connected.find((gamepad) => gamepad.id === preferredId) ?? null;
  }
  return connected[0] ?? null;
}

export function gamepadInputState(gamepad: GamepadSample): Map<string, number> {
  const next = new Map<string, number>();
  for (const [indexText, token] of Object.entries(GAMEPAD_BUTTON_INPUTS)) {
    const button = gamepad.buttons[Number(indexText)];
    next.set(token, button?.pressed ? 32767 : 0);
  }

  const horizontal = normalizedAxis(gamepad.axes[0] ?? 0);
  const vertical = normalizedAxis(gamepad.axes[1] ?? 0);
  next.set("P1_AD_STICK_X", horizontal);
  next.set("P1_AD_STICK_Y", vertical);
  next.set("P1_JOYSTICK_LEFT", horizontal <= -GAMEPAD_DIGITAL_THRESHOLD * 32768 ? 32767 : 0);
  next.set("P1_JOYSTICK_RIGHT", horizontal >= GAMEPAD_DIGITAL_THRESHOLD * 32767 ? 32767 : 0);
  next.set("P1_JOYSTICK_UP", vertical <= -GAMEPAD_DIGITAL_THRESHOLD * 32768 ? 32767 : 0);
  next.set("P1_JOYSTICK_DOWN", vertical >= GAMEPAD_DIGITAL_THRESHOLD * 32767 ? 32767 : 0);
  return next;
}

export function combineInputState(
  keyboard: ReadonlyMap<string, number>,
  gamepad: ReadonlyMap<string, number>,
): Map<string, number> {
  const combined = new Map<string, number>();
  for (const source of [keyboard, gamepad]) {
    for (const [token, value] of source) {
      const current = combined.get(token) ?? 0;
      if (Math.abs(value) >= Math.abs(current)) combined.set(token, value);
    }
  }
  return combined;
}

export function diffInputState(
  desired: ReadonlyMap<string, number>,
  accepted: ReadonlyMap<string, number>,
): MameInputUpdate[] {
  const tokens = new Set([...desired.keys(), ...accepted.keys()]);
  const updates: MameInputUpdate[] = [];
  for (const token of tokens) {
    const desiredValue = desired.get(token) ?? 0;
    if (desiredValue !== (accepted.get(token) ?? 0)) {
      updates.push({ token, value: desiredValue });
    }
  }
  return updates;
}
