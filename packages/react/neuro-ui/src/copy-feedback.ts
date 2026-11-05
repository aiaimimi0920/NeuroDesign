import { useEffect, useRef, useState } from "react";

/** Clipboard feedback only; permission checks and secret retrieval belong to the caller. */
export function useCopyFeedback(onCopy: () => Promise<void>, duration: number, resetError = false) {
  const [state, setState] = useState<"idle" | "copying" | "copied" | "error">("idle");
  const busy = useRef(false);
  const generation = useRef(0);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => () => { generation.current++; clearTimeout(timer.current); }, []);
  async function copy() {
    if (busy.current) return;
    busy.current = true;
    const current = generation.current;
    clearTimeout(timer.current);
    setState("copying");
    let copied = false;
    try {
      await onCopy();
      if (current !== generation.current) return;
      setState("copied");
      copied = true;
    } catch {
      if (current !== generation.current) return;
      setState("error");
    } finally {
      busy.current = false;
    }
    if (copied || resetError) timer.current = setTimeout(() => setState("idle"), duration);
  }
  return { state, copy };
}
