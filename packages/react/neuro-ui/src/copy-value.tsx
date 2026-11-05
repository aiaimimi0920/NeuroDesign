import { IconButton } from "./actions";
import { Icon } from "./icons";
import { useCopyFeedback } from "./copy-feedback";

export interface CopyValueProps {
  value: string;
  label: string;
  displayValue?: string;
  writeText?: (value: string) => Promise<void>;
  copiedLabel?: string;
  errorLabel?: string;
}
async function browserWriteText(value: string) {
  if (!navigator.clipboard?.writeText) throw new Error("Clipboard API is unavailable");
  await navigator.clipboard.writeText(value);
}
export function CopyValue({ value, label, displayValue, writeText = browserWriteText, copiedLabel = "已复制", errorLabel = "复制失败，请选取内容手动复制" }: CopyValueProps) {
  const { state, copy } = useCopyFeedback(() => writeText(value), 1800);
  return <div className="n-copy-value" data-state={state}>
    <code>{displayValue ?? value}</code>
    <IconButton label={label} compact variant="quiet" onClick={copy} disabled={state === "copying"} aria-busy={state === "copying" || undefined}>
      <Icon name={state === "copied" ? "check" : "copy"} />
    </IconButton>
    <span role="status" className={state === "error" ? "n-copy-error" : "n-sr-only"}>{state === "copied" ? copiedLabel : state === "error" ? errorLabel : ""}</span>
  </div>;
}
