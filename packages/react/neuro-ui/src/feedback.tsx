import { createContext, useCallback, useContext, useEffect, useRef, useState, type HTMLAttributes, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { Icon, type IconName } from "./icons";
import { cx, useNeuroTheme } from "./theme";

export type Tone = "neutral" | "success" | "warning" | "info" | "danger";
const toneIcons: Record<Tone, IconName> = { neutral: "info", success: "check", warning: "warning", info: "info", danger: "close" };
export function Badge({ tone = "neutral", children, className, ...props }: HTMLAttributes<HTMLSpanElement> & { tone?: Tone }) {
  return <span {...props} className={cx("n-badge", `n-tone-${tone}`, className)}><Icon name={toneIcons[tone]} />{children}</span>;
}
export function Notice({ tone = "info", children, className, ...props }: HTMLAttributes<HTMLDivElement> & { tone?: Tone }) {
  return <div role={tone === "danger" ? "alert" : "status"} {...props} className={cx("n-notice", `n-tone-${tone}`, className)}><Icon name={toneIcons[tone]} /><div>{children}</div></div>;
}
export function Progress({ label, value, max = 100 }: { label: string; value?: number; max?: number }) {
  return <progress className="n-progress" aria-label={label} value={value} max={max} />;
}
export function EmptyState({ title, compact, children, className, ...props }: HTMLAttributes<HTMLDivElement> & { title: string; compact?: boolean }) {
  return <div {...props} className={cx("n-empty", compact && "n-empty--compact", className)}><Icon name="folder" /><strong>{title}</strong>{children}</div>;
}

export type ToastInput = { message: string; tone?: Exclude<Tone, "neutral">; duration?: number };
export type ToastEntry = ToastInput & { id: number };
const ToastContext = createContext<((toast: ToastInput) => void) | null>(null);

function ToastItem({ entry, dismiss, closeLabel }: { entry: ToastEntry; dismiss: (id: number) => void; closeLabel: string }) {
  const tone = entry.tone ?? "info";
  const [exiting, setExiting] = useState(false);
  const exitTimer = useRef<number | undefined>(undefined);
  const dismissItem = useCallback(() => {
    if (exitTimer.current !== undefined) return;
    setExiting(true);
    exitTimer.current = window.setTimeout(() => dismiss(entry.id), 220);
  }, [dismiss, entry.id]);
  useEffect(() => {
    const timer = window.setTimeout(dismissItem, entry.duration ?? (tone === "danger" ? 5200 : tone === "warning" ? 4200 : 3200));
    return () => window.clearTimeout(timer);
  }, [entry.duration, tone, dismissItem]);
  useEffect(() => () => { window.clearTimeout(exitTimer.current); exitTimer.current = undefined; }, []);
  return <button type="button" role={tone === "danger" ? "alert" : "status"} aria-live={tone === "danger" ? "assertive" : "polite"}
    aria-label={`${entry.message}，${closeLabel}`} onClick={dismissItem} data-exiting={exiting || undefined} className={`n-toast n-tone-${tone}`}>
    <Icon name={toneIcons[tone]} /><span>{entry.message}</span>
    <Icon name="close" />
  </button>;
}

export interface ToastViewportProps {
  entries: readonly ToastEntry[];
  onRemove: (id: number) => void;
  label?: string;
  closeLabel?: string;
}
/** Applications own publication; the component owns presentation and timer cleanup. */
export function ToastViewport({ entries, onRemove, label = "通知", closeLabel = "关闭通知" }: ToastViewportProps) {
  const { portal } = useNeuroTheme();
  return portal ? createPortal(<div className="n-toast-stack" aria-label={label}>{entries.map(entry => <ToastItem key={entry.id} entry={entry} dismiss={onRemove} closeLabel={closeLabel} />)}</div>, portal) : null;
}

/** Per-tree feedback state, never a process-global bus or a business task queue. */
export function ToastProvider({ children, closeLabel = "关闭通知" }: { children: ReactNode; closeLabel?: string }) {
  const [entries, setEntries] = useState<ToastEntry[]>([]);
  const nextId = useRef(0);
  const notify = useCallback((entry: ToastInput) => {
    const id = ++nextId.current;
    setEntries(current => [...current, { ...entry, id }]);
  }, []);
  const dismiss = useCallback((id: number) => setEntries(current => current.filter(entry => entry.id !== id)), []);
  return <ToastContext.Provider value={notify}>{children}
    <ToastViewport entries={entries} onRemove={dismiss} closeLabel={closeLabel} />
  </ToastContext.Provider>;
}
export function useToast() {
  const notify = useContext(ToastContext);
  if (!notify) throw new Error("useToast requires a ToastProvider");
  return notify;
}
