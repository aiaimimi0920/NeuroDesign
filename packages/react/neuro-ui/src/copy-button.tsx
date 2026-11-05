import type { ReactNode } from "react";
import { Button, type ButtonProps } from "./actions";
import { useCopyFeedback } from "./copy-feedback";
import { Icon } from "./icons";

export interface CopyButtonProps extends Omit<ButtonProps, "onClick" | "children" | "loading"> {
  label: string;
  children?: ReactNode;
  onCopy: () => Promise<void>;
  copiedLabel?: string;
  errorLabel?: string;
  feedbackDuration?: number;
}
export function CopyButton({ label, children, onCopy, copiedLabel = "已复制", errorLabel = "复制失败", feedbackDuration = 1800, ...props }: CopyButtonProps) {
  const { state, copy } = useCopyFeedback(onCopy, feedbackDuration, true);
  return <Button {...props} aria-label={label} title={state === "copied" ? copiedLabel : state === "error" ? errorLabel : label}
    data-copy-state={state} loading={state === "copying"} onClick={copy}>
    <Icon name={state === "copied" ? "check" : state === "error" ? "close" : "copy"} />
    <span className="n-copy-caption"><span className="n-copy-rest" aria-hidden={state === "error"}>{children ?? label}</span><span className="n-copy-failure" aria-hidden="true">{errorLabel}</span></span>
    <span className="n-sr-only" role="status">{state === "copied" ? copiedLabel : state === "error" ? errorLabel : ""}</span>
  </Button>;
}
