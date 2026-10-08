import { useEffect, useRef, useState } from "react";

import { getMameGameFrame } from "../backend/commands";
import { errorMessage, isAppErrorEnvelope } from "../backend/errors";
import type { SessionSnapshot } from "../backend/types";
import { bgrxToRgba, parseGameFrame, type GameFrame } from "./frameProtocol";
import "./GameSurface.css";

type FrameState = "waitingRuntime" | "waitingFrame" | "active" | "stalled" | "error";

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
  stagingContext.putImageData(
    new ImageData(bgrxToRgba(frame), frame.width, frame.height),
    0,
    0,
  );

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
          aria-label="MAME gameplay video"
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
      </footer>
    </section>
  );
}
