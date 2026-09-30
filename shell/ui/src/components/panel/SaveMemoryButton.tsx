import { useEffect, useRef, useState, type ReactElement } from "react";

import { invokeTauriAsync } from "../../tauri";
import { FOCUS_RING } from "../settings/primitives";

/** The OD bookmark mark (`capture-and-ask.frag`), sized by `.btn svg`. */
function BookmarkGlyph(): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M6 4h12v16l-6-4-6 4Z" />
    </svg>
  );
}

type SaveState = "idle" | "saving" | "saved" | "duplicate";

/** How long the Undo affordance stays up after a save. */
const UNDO_MS = 4000;

/**
 * "Save to memory" — stores the answer in the semantic memory store
 * (RFC-0011 F-10) with the capture as its provenance. An identical body
 * returns the existing note ("Already saved"); a fresh save offers a 4 s
 * Undo, which archives the note it just created (nothing is hard-deleted).
 */
export default function SaveMemoryButton({
  answer,
  captureId,
}: {
  answer: string;
  captureId?: string;
}) {
  const [state, setState] = useState<SaveState>("idle");
  const [savedId, setSavedId] = useState<string | null>(null);
  const undoTimer = useRef<number | null>(null);

  useEffect(
    () => () => {
      if (undoTimer.current !== null) window.clearTimeout(undoTimer.current);
    },
    [],
  );

  const save = async () => {
    if (state !== "idle" || answer.trim().length === 0) return;
    setState("saving");
    try {
      const result = (await invokeTauriAsync("memory_save_semantic", {
        title: "Saved answer",
        body: answer,
        tags: [],
        sourceRefs: captureId ? [captureId] : [],
      })) as { id?: string; created?: boolean } | null;
      if (!result?.id) {
        setState("idle");
        return;
      }
      setSavedId(result.id);
      if (result.created === false) {
        setState("duplicate");
        return;
      }
      setState("saved");
      undoTimer.current = window.setTimeout(() => {
        setState("idle");
        setSavedId(null);
      }, UNDO_MS);
    } catch {
      setState("idle");
    }
  };

  const undo = async () => {
    if (savedId === null) return;
    if (undoTimer.current !== null) window.clearTimeout(undoTimer.current);
    await invokeTauriAsync("memory_archive", { id: savedId });
    setState("idle");
    setSavedId(null);
  };

  const label =
    state === "saving"
      ? "Saving…"
      : state === "saved"
        ? "Saved"
        : state === "duplicate"
          ? "Already saved"
          : "Save to memory";

  return (
    <>
      <button
        type="button"
        title={state === "duplicate" ? "This answer is already in memory" : undefined}
        onClick={save}
        disabled={state === "saving" || state === "saved" || state === "duplicate"}
        className={`panel-save-memory ${FOCUS_RING}`}
      >
        <BookmarkGlyph />
        <span>{label}</span>
      </button>
      {state === "saved" ? (
        <button
          type="button"
          onClick={undo}
          className={`font-mono text-[11px] leading-[1.4] text-muted-foreground transition-colors duration-[120ms] hover:text-foreground ${FOCUS_RING}`}
        >
          Undo
        </button>
      ) : null}
    </>
  );
}
