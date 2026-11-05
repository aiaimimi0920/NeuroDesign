import { forwardRef, type AnchorHTMLAttributes, type ButtonHTMLAttributes, type ReactNode } from "react";
import { cx } from "./theme";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "quiet" | "danger" | "positive";
  compact?: boolean;
  loading?: boolean;
  wrap?: boolean;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = "secondary", compact, loading, wrap, disabled, className, children, type = "button", ...props }, ref,
) {
  return <button {...props} ref={ref} type={type} disabled={disabled || loading} aria-busy={loading || props["aria-busy"] || undefined}
    className={cx("n-button", `n-button--${variant}`, compact && "n-compact", loading && "n-loading", wrap && "n-button-wrap", className)}>
    <span className="n-button-label">{children}</span>
    {loading ? <span className="n-spinner" aria-hidden="true" /> : null}
  </button>;
});

export interface ButtonLinkProps extends Omit<AnchorHTMLAttributes<HTMLAnchorElement>, "href">, Pick<ButtonProps, "variant" | "compact" | "wrap"> {
  href: string;
}

/** Navigation stays a native link. The caller owns URL policy and unavailable states. */
export const ButtonLink = forwardRef<HTMLAnchorElement, ButtonLinkProps>(function ButtonLink(
  { variant = "secondary", compact, wrap, className, children, ...props }, ref,
) {
  return <a {...props} ref={ref} className={cx("n-button", "n-button-link", `n-button--${variant}`, compact && "n-compact", wrap && "n-button-wrap", className)}>
    <span className="n-button-label">{children}</span>
  </a>;
});

export interface IconButtonProps extends Omit<ButtonProps, "aria-label" | "children"> {
  label: string;
  children: ReactNode;
}

export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(function IconButton(
  { label, className, ...props }, ref,
) {
  return <Button {...props} ref={ref} aria-label={label} className={cx("n-icon-button", className)} />;
});
