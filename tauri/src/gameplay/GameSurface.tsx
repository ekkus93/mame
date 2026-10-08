import { useEffect, useRef, useState } from "react";

import {
  getControllerProfileConfiguration,
  getMameFrameMetrics,
  getMameGameFrame,
  setMameInputs,
} from "../backend/commands";
import { errorMessage, isAppErrorEnvelope } from "../backend/errors";
import type { SessionSnapshot } from "../backend/types";
import { bgrxToRgba, parseGameFrame, type GameFrame } from "./frameProtocol";
import {
  combineInputState,
  diffInputState,
  gamepadInputState,
  KEYBOARD_INPUTS,
  selectStandardGamepad,
} from "./gameplayInput";
import "./GameSurface.css";

type FrameState = "starting" | "waitingFrame" | "active" | "stalled" | "stopping" | "error";

type GameSurfaceProps = {
  session: SessionSnapshot;
  paused: boolean;
  muted: boolean;
  canCommand: boolean;
  stopping: boolean;
  onPause: () => void;
  onResume: () => void;
  onReset: () => void;
  onToggleMute: () => void;
  onStop: () => void;
};

const FIRST_FRAME_TIMEOUT_MS = 5_000;
const FRAME_STALL_TIMEOUT_MS = 2_000;

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

export function GameSurface({
  session,
  paused,
  muted,
  canCommand,
  stopping,
  onPause,
  onResume,
  onReset,
  onToggleMute,
  onStop,
}: GameSurfaceProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const stagingRef = useRef<HTMLCanvasElement | null>(null);
  const surfaceRef = useRef<HTMLElement | null>(null);
  const keyboardInputsRef = useRef(new Map<string, number>());
  const gamepadInputsRef = useRef(new Map<string, number>());
  const acceptedInputsRef = useRef(new Map<string, number>());
  const inputOwnedRef = useRef(false);
  const inputInFlightRef = useRef(false);
  const presentationAckRef = useRef<{
    sequence: string;
    durationUs: number;
  } | null>(null);
  const [frameState, setFrameState] = useState<FrameState>(
    session.state === "running"
      ? "waitingFrame"
      : session.state === "stopping"
        ? "stopping"
        : "starting",
  );
  const [failure, setFailure] = useState<string | null>(null);
  const [smoothScaling, setSmoothScaling] = useState(false);
  const [frameSummary, setFrameSummary] = useState<string>("No frame presented yet.");
  const [preferredGamepadId, setPreferredGamepadId] = useState<string | null>(null);
  const [controllerSummary, setControllerSummary] = useState(
    "Gamepad: first connected W3C-standard controller",
  );

  useEffect(() => {
    let disposed = false;
    void getControllerProfileConfiguration({
      scope: { kind: "machine", shortName: session.machine },
    })
      .then((configuration) => {
        if (disposed) return;
        const profile = configuration.effectiveProfile;
        if (
          profile?.targetDevice.kind === "browserGamepadId" &&
          profile.targetDevice.reportedMapping === "standard" &&
          profile.mappingProvenance.kind === "browserStandardGamepad"
        ) {
          setPreferredGamepadId(profile.targetDevice.value);
          setControllerSummary(`Gamepad profile: ${profile.name}`);
        } else {
          setPreferredGamepadId(null);
          setControllerSummary("Gamepad: first connected W3C-standard controller");
        }
      })
      .catch(() => {
        if (!disposed) {
          setPreferredGamepadId(null);
          setControllerSummary("Gamepad profile unavailable; using first standard controller");
        }
      });

    return () => {
      disposed = true;
    };
  }, [session.machine]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    if (session.state !== "running") {
      setFrameState(session.state === "stopping" ? "stopping" : "starting");
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
        const acknowledgement = presentationAckRef.current;
        presentationAckRef.current = null;
        const raw = await getMameGameFrame({
          sessionId: session.sessionId,
          presentedSequence: acknowledgement?.sequence,
          presentationDurationUs: acknowledgement?.durationUs,
        });
        if (disposed) return;

        const frame = parseGameFrame(raw, session.sessionId);
        if (frame.sequence <= lastSequence) {
          throw new Error("The MAME gameplay frame sequence moved backwards.");
        }
        lastSequence = frame.sequence;
        const presentationStartedAt = performance.now();
        drawFrame(canvas, staging, frame, smoothScaling);
        const presentationDurationUs = Math.max(
          0,
          Math.round((performance.now() - presentationStartedAt) * 1_000),
        );
        presentationAckRef.current = {
          sequence: frame.sequence.toString(),
          durationUs: presentationDurationUs,
        };

        const now = performance.now();
        if (firstFrameAt === null) firstFrameAt = now;
        lastPresentedAt = now;
        setFrameState("active");
        setFailure(null);

        if (now - lastSummaryAt >= 500) {
          lastSummaryAt = now;
          const orientation =
            frame.orientationDegrees === 0 ? "" : ` · rotated ${frame.orientationDegrees}°`;
          const baseSummary = `${frame.width}×${frame.height}${orientation} · frame ${frame.sequence.toString()}`;
          setFrameSummary(baseSummary);
          void getMameFrameMetrics({ sessionId: session.sessionId })
            .then((metrics) => {
              if (disposed) return;
              const age = metrics.latestAgeMs === null ? "age —" : `age ${metrics.latestAgeMs}ms`;
              setFrameSummary(
                `${baseSummary} · recv ${metrics.received} · drop ${metrics.dropped} · ` +
                  `presented ${metrics.presented} · ${age}`,
              );
            })
            .catch(() => undefined);
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
      presentationAckRef.current = null;
      window.cancelAnimationFrame(animationFrame);
    };
  }, [session.sessionId, session.state, smoothScaling]);

  useEffect(() => {
    let disposed = false;
    let animationFrame = 0;
    const acceptedInputs = acceptedInputsRef.current;

    const releaseDesiredInputs = () => {
      keyboardInputsRef.current.clear();
      gamepadInputsRef.current.clear();
    };

    const updateGamepad = () => {
      if (!inputOwnedRef.current || typeof navigator.getGamepads !== "function") {
        gamepadInputsRef.current = new Map();
        return;
      }
      const gamepad = selectStandardGamepad(
        Array.from(navigator.getGamepads()),
        preferredGamepadId,
      );
      gamepadInputsRef.current = gamepad ? gamepadInputState(gamepad) : new Map();
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
      const release = Array.from(acceptedInputs.keys()).map((token) => ({
        token,
        value: 0,
      }));
      acceptedInputs.clear();
      if (release.length > 0) {
        void setMameInputs({ sessionId: session.sessionId, updates: release.slice(0, 32) }).catch(
          () => undefined,
        );
      }
    };
  }, [preferredGamepadId, session.sessionId, session.state]);

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
    starting: "Starting MAME runtime…",
    waitingFrame: "Waiting for first gameplay frame…",
    active: "Gameplay active",
    stalled: "Gameplay video stalled",
    stopping: "Stopping MAME…",
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
          <button type="button" disabled={!canCommand || paused} onClick={onPause}>
            Pause
          </button>
          <button type="button" disabled={!canCommand || !paused} onClick={onResume}>
            Resume
          </button>
          <button type="button" disabled={!canCommand} onClick={onReset}>
            Reset
          </button>
          <button type="button" disabled={!canCommand} onClick={onToggleMute}>
            {muted ? "Unmute" : "Mute"}
          </button>
          <button type="button" disabled={!canCommand || stopping} onClick={onStop}>
            {stopping ? "Stopping…" : "Stop"}
          </button>
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
        <span>
          Focus game: arrows · Z/X/C/V · Enter start · 5 coin · {controllerSummary}
        </span>
      </footer>
    </section>
  );
}
