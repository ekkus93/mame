import { useEffect, useRef, useState } from "react";

import { getMameGameFrame, setMameInputs } from "../backend/commands";
import { errorMessage, isAppErrorEnvelope } from "../backend/errors";
import type { MameInputUpdate, SessionSnapshot } from "../backend/types";
import { bgrxToRgba, parseGameFrame, type GameFrame } from "./frameProtocol";
import "./GameSurface.css";

type FrameState = "waitingRuntime" | "waitingFrame" | "active" | "stalled" | "error";

const FIRST_FRAME_TIMEOUT_MS = 5_000;
const FRAME_STALL_TIMEOUT_MS = 2_000;

const KEYBOARD_INPUTS: Readonly<Record<string, string>> = {
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

const GAMEPAD_DEADZONE = 0.12;
const GAMEPAD_DIGITAL_THRESHOLD = 0.5;

function normalizedAxis(value: number): number {
  if (!Number.isFinite(value) || Math.abs(value) < GAMEPAD_DEADZONE) return 0;
  return Math.max(-32768, Math.min(32767, Math.round(value * 32767)));
}

function combineInputState(
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

function diffInputState(
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

function drawFrame(
  canvas: HTMLCanvasElement,
  staging: HTMLCanvasElement,
  frame: GameFrame,
  smoothing: boolean,
) {
  staging.width = frame.width;
  staging.height = frame.height;
  const stagingContext = staging.getContext("2d", { alpha: false });
  if (!stagingContext) {
    throw new Error("The browser could not create the gameplay staging canvas.");
  }
  stagingContext.putImageData(new ImageData(bgrxToRgba(frame), frame.width, frame.height), 0, 0);

  const rotated = frame.orientationDegrees === 90 || frame.orientationDegrees === 270;
  const displayWidth = rotated ? frame.height : frame.width;
  const displayHeight = rotated ? frame.width : frame.height;
  if (canvas.width !== displayWidth) canvas.width = displayWidth;
  if (canvas.height !== displayHeight) canvas.height = displayHeight;

  const context = canvas.getContext("2d", { alpha: false });
  if (!context) {
    throw new Error("The browser could not create the gameplay canvas.");
  }

  context.save();
  context.setTransform(1, 0, 0, 1, 0, 0);
  context.clearRect(0, 0, displayWidth, displayHeight);
  context.imageSmoothingEnabled = smoothing;

  if (frame.flipX || frame.flipY) {
    context.translate(frame.flipX ? displayWidth : 0, frame.flipY ? displayHeight : 0);
    context.scale(frame.flipX ? -1 : 1, frame.flipY ? -1 : 1);
  }

  switch (frame.orientationDegrees) {
    case 0:
      break;
    case 90:
      context.translate(displayWidth, 0);
      context.rotate(Math.PI / 2);
      break;
    case 180:
      context.translate(displayWidth, displayHeight);
      context.rotate(Math.PI);
      break;
    case 270:
      context.translate(0, displayHeight);
      context.rotate(-Math.PI / 2);
      break;
  }

  context.drawImage(staging, 0, 0);
  context.restore();
}

export function GameSurface({ session }: { session: SessionSnapshot }) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const stagingRef = useRef<HTMLCanvasElement | null>(null);
  const surfaceRef = useRef<HTMLElement | null>(null);
  const keyboardInputsRef = useRef(new Map<string, number>());
  const gamepadInputsRef = useRef(new Map<string, number>());
  const acceptedInputsRef = useRef(new Map<string, number>());
  const inputOwnedRef = useRef(false);
  const inputInFlightRef = useRef(false);
  const [frameState, setFrameState] = useState<FrameState>(
    session.state === "running" ? "waitingFrame" : "waitingRuntime",
  );
  const [failure, setFailure] = useState<string | null>(null);
  const [smoothScaling, setSmoothScaling] = useState(false);
  const [frameSummary, setFrameSummary] = useState<string>("No frame presented yet.");

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    if (session.state !== "running") {
      setFrameState("waitingRuntime");
      setFailure(null);
      return;
    }

    let disposed = false;
    let animationFrame = 0;
    let firstFrameAt: number | null = null;
    const startedAt = performance.now();
    let lastPresentedAt = startedAt;
    let lastSummaryAt = 0;
    let lastSequence = -1n;

    if (!stagingRef.current) {
      stagingRef.current = document.createElement("canvas");
    }
    const staging = stagingRef.current;

    setFrameState("waitingFrame");
    setFailure(null);

    const poll = async () => {
      if (disposed) return;

      try {
        const raw = await getMameGameFrame({ sessionId: session.sessionId });
        if (disposed) return;

        const frame = parseGameFrame(raw, session.sessionId);
        if (frame.sequence <= lastSequence) {
          throw new Error("The MAME gameplay frame sequence moved backwards.");
        }
        lastSequence = frame.sequence;
        drawFrame(canvas, staging, frame, smoothScaling);

        const now = performance.now();
        if (firstFrameAt === null) firstFrameAt = now;
        lastPresentedAt = now;
        setFrameState("active");
        setFailure(null);

        if (now - lastSummaryAt >= 500) {
          lastSummaryAt = now;
          const orientation =
            frame.orientationDegrees === 0 ? "" : ` · rotated ${frame.orientationDegrees}°`;
          setFrameSummary(
            `${frame.width}×${frame.height}${orientation} · frame ${frame.sequence.toString()}`,
          );
        }
      } catch (error: unknown) {
        if (disposed) return;
        const now = performance.now();
        if (isAppErrorEnvelope(error) && error.code === "MAME_FRAME_NOT_READY") {
          if (firstFrameAt === null && now - startedAt >= FIRST_FRAME_TIMEOUT_MS) {
            setFrameState("stalled");
            setFailure("MAME is running, but no gameplay frame has arrived yet.");
          } else if (firstFrameAt !== null && now - lastPresentedAt >= FRAME_STALL_TIMEOUT_MS) {
            setFrameState("stalled");
            setFailure("The MAME gameplay frame stream has stalled.");
          }
        } else {
          setFrameState("error");
          setFailure(errorMessage(error));
        }
      }

      if (!disposed) {
        animationFrame = window.requestAnimationFrame(() => {
          void poll();
        });
      }
    };

    void poll();

    return () => {
      disposed = true;
      window.cancelAnimationFrame(animationFrame);
    };
  }, [session.sessionId, session.state, smoothScaling]);

  useEffect(() => {
    let disposed = false;
    let animationFrame = 0;

    const releaseDesiredInputs = () => {
      keyboardInputsRef.current.clear();
      gamepadInputsRef.current.clear();
    };

    const updateGamepad = () => {
      const next = new Map<string, number>();
      if (inputOwnedRef.current && typeof navigator.getGamepads === "function") {
        const gamepad = Array.from(navigator.getGamepads()).find(
          (candidate): candidate is Gamepad => candidate !== null && candidate.connected,
        );
        if (gamepad) {
          for (const [indexText, token] of Object.entries(GAMEPAD_BUTTON_INPUTS)) {
            const button = gamepad.buttons[Number(indexText)];
            next.set(token, button?.pressed ? 32767 : 0);
          }
          const horizontal = normalizedAxis(gamepad.axes[0] ?? 0);
          const vertical = normalizedAxis(gamepad.axes[1] ?? 0);
          next.set("P1_AD_STICK_X", horizontal);
          next.set("P1_AD_STICK_Y", vertical);
          next.set(
            "P1_JOYSTICK_LEFT",
            horizontal <= -GAMEPAD_DIGITAL_THRESHOLD * 32768 ? 32767 : 0,
          );
          next.set(
            "P1_JOYSTICK_RIGHT",
            horizontal >= GAMEPAD_DIGITAL_THRESHOLD * 32767 ? 32767 : 0,
          );
          next.set("P1_JOYSTICK_UP", vertical <= -GAMEPAD_DIGITAL_THRESHOLD * 32768 ? 32767 : 0);
          next.set("P1_JOYSTICK_DOWN", vertical >= GAMEPAD_DIGITAL_THRESHOLD * 32767 ? 32767 : 0);
        }
      }
      gamepadInputsRef.current = next;
    };

    const pump = () => {
      if (disposed) return;
      updateGamepad();

      if (!inputInFlightRef.current && session.state === "running") {
        const desired = combineInputState(keyboardInputsRef.current, gamepadInputsRef.current);
        const updates = diffInputState(desired, acceptedInputsRef.current);
        if (updates.length > 0) {
          inputInFlightRef.current = true;
          void setMameInputs({
            sessionId: session.sessionId,
            updates: updates.slice(0, 32),
          })
            .then((result) => {
              if (!result.accepted) return;
              for (const update of updates.slice(0, 32)) {
                if (update.value === 0) acceptedInputsRef.current.delete(update.token);
                else acceptedInputsRef.current.set(update.token, update.value);
              }
            })
            .catch(() => {
              // Session teardown or runtime-control failure is reflected by the
              // session/gameplay state; input remains best-effort and bounded.
            })
            .finally(() => {
              inputInFlightRef.current = false;
            });
        }
      }

      animationFrame = window.requestAnimationFrame(pump);
    };

    const handleFullscreenChange = () => {
      releaseDesiredInputs();
      if (document.fullscreenElement === surfaceRef.current) {
        canvasRef.current?.focus();
      }
    };
    document.addEventListener("fullscreenchange", handleFullscreenChange);
    animationFrame = window.requestAnimationFrame(pump);

    return () => {
      disposed = true;
      window.cancelAnimationFrame(animationFrame);
      document.removeEventListener("fullscreenchange", handleFullscreenChange);
      inputOwnedRef.current = false;
      releaseDesiredInputs();
      const release = Array.from(acceptedInputsRef.current.keys()).map((token) => ({
        token,
        value: 0,
      }));
      acceptedInputsRef.current.clear();
      if (release.length > 0) {
        void setMameInputs({ sessionId: session.sessionId, updates: release.slice(0, 32) }).catch(
          () => undefined,
        );
      }
    };
  }, [session.sessionId, session.state]);

  const handleGameplayFocus = () => {
    inputOwnedRef.current = true;
  };

  const handleGameplayBlur = () => {
    inputOwnedRef.current = false;
    keyboardInputsRef.current.clear();
    gamepadInputsRef.current.clear();
  };

  const handleGameplayKeyDown = (event: React.KeyboardEvent<HTMLCanvasElement>) => {
    const token = KEYBOARD_INPUTS[event.code];
    if (!token || event.repeat) return;
    event.preventDefault();
    keyboardInputsRef.current.set(token, 32767);
  };

  const handleGameplayKeyUp = (event: React.KeyboardEvent<HTMLCanvasElement>) => {
    const token = KEYBOARD_INPUTS[event.code];
    if (!token) return;
    event.preventDefault();
    keyboardInputsRef.current.delete(token);
  };

  const toggleFullscreen = () => {
    const surface = surfaceRef.current;
    if (!surface) return;
    if (document.fullscreenElement) {
      void document.exitFullscreen();
    } else {
      void surface.requestFullscreen();
    }
  };

  const statusLabel = {
    waitingRuntime: "Starting MAME runtime…",
    waitingFrame: "Waiting for first gameplay frame…",
    active: "Gameplay active",
    stalled: "Gameplay video stalled",
    error: "Gameplay video unavailable",
  }[frameState];

  return (
    <section
      ref={surfaceRef}
      className={`game-surface game-surface-${frameState}`}
      aria-labelledby="game-surface-heading"
    >
      <header className="game-surface-toolbar">
        <div>
          <p className="eyebrow">In-app gameplay</p>
          <h2 id="game-surface-heading">
            {session.machine}
            {session.software ? ` · ${session.software}` : ""}
          </h2>
        </div>
        <div className="game-surface-actions">
          <label>
            Scaling
            <select
              value={smoothScaling ? "smooth" : "nearest"}
              onChange={(event) => setSmoothScaling(event.target.value === "smooth")}
            >
              <option value="nearest">Nearest</option>
              <option value="smooth">Smooth</option>
            </select>
          </label>
          <button type="button" onClick={toggleFullscreen}>
            {document.fullscreenElement ? "Exit fullscreen" : "Fullscreen"}
          </button>
        </div>
      </header>
      <div className="game-surface-stage">
        <canvas
          ref={canvasRef}
          className={smoothScaling ? "is-smooth" : "is-nearest"}
          tabIndex={0}
          aria-label="MAME gameplay video. Focus this surface to control the game."
          onFocus={handleGameplayFocus}
          onBlur={handleGameplayBlur}
          onKeyDown={handleGameplayKeyDown}
          onKeyUp={handleGameplayKeyUp}
        />
        {frameState !== "active" && (
          <div className="game-surface-overlay" role={frameState === "error" ? "alert" : "status"}>
            <strong>{statusLabel}</strong>
            {failure && <span>{failure}</span>}
          </div>
        )}
      </div>
      <footer className="game-surface-status" aria-live="polite">
        <span>{statusLabel}</span>
        <span>{frameSummary}</span>
        <span>Focus game: arrows · Z/X/C/V · Enter start · 5 coin · gamepad supported</span>
      </footer>
    </section>
  );
}
