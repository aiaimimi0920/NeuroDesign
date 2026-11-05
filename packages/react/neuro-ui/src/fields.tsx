import { cloneElement, forwardRef, useId, type InputHTMLAttributes, type ReactElement, type SelectHTMLAttributes, type TextareaHTMLAttributes } from "react";
import { cx } from "./theme";

type FieldControl = { id?: string; "aria-describedby"?: string; "aria-invalid"?: boolean | "true" | "false" };
export function Field({ label, error, description, children }: {
  label: string;
  error?: string;
  description?: string;
  children: ReactElement<FieldControl>;
}) {
  const generated = useId();
  const id = children.props.id ?? generated;
  const message = error || description;
  const describedBy = [children.props["aria-describedby"], message ? `${id}-message` : undefined].filter(Boolean).join(" ") || undefined;
  return <div className="n-field">
    <label htmlFor={id}>{label}</label>
    {cloneElement(children, { id, "aria-invalid": error ? true : children.props["aria-invalid"], "aria-describedby": describedBy })}
    {message ? <span id={`${id}-message`} className={error ? "n-field-error" : "n-muted"} role={error ? "alert" : undefined}>{message}</span> : null}
  </div>;
}

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> { compact?: boolean }
export const Input = forwardRef<HTMLInputElement, InputProps>(function Input({ compact, className, ...props }, ref) {
  return <input {...props} ref={ref} className={cx("n-input", compact && "n-compact", className)} />;
});
export interface TextareaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> { compact?: boolean }
export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(function Textarea({ compact, className, ...props }, ref) {
  return <textarea {...props} ref={ref} className={cx("n-input", compact && "n-compact", className)} />;
});
export interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> { compact?: boolean }
export const Select = forwardRef<HTMLSelectElement, SelectProps>(function Select({ compact, className, children, ...props }, ref) {
  return <select {...props} ref={ref} className={cx("n-input", "n-select", compact && "n-compact", className)}>{children}</select>;
});

export const Range = forwardRef<HTMLInputElement, Omit<InputHTMLAttributes<HTMLInputElement>, "type">>(function Range({ className, ...props }, ref) {
  return <input {...props} ref={ref} type="range" className={cx("n-range", className)} />;
});
