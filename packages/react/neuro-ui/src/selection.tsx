import { forwardRef, useCallback, useId, type ForwardedRef, type InputHTMLAttributes, type ReactNode } from "react";
import { cx } from "./theme";

export interface CheckboxProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "type"> {
  label?: ReactNode;
  indeterminate?: boolean;
}
function assignRef<T>(ref: ForwardedRef<T>, value: T | null) {
  if (typeof ref === "function") ref(value);
  else if (ref) ref.current = value;
}
export const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>(function Checkbox(
  { label, indeterminate = false, className, ...props }, ref,
) {
  const attach = useCallback((element: HTMLInputElement | null) => {
    if (element) element.indeterminate = indeterminate;
    assignRef(ref, element);
  }, [ref, indeterminate]);
  return <label className={cx("n-check", className)}><input {...props} ref={attach} type="checkbox" />{label ? <span>{label}</span> : null}</label>;
});

export const Switch = forwardRef<HTMLInputElement, Omit<CheckboxProps, "indeterminate">>(function Switch(
  { label, className, ...props }, ref,
) {
  return <label className={cx("n-switch", className)}><input {...props} ref={ref} type="checkbox" role="switch" />{label ? <span>{label}</span> : null}</label>;
});

export interface ChoiceOption<Value extends string = string> { value: Value; label: ReactNode; disabled?: boolean }
export interface RadioGroupProps<Value extends string = string> {
  label: string;
  name: string;
  options: readonly ChoiceOption<Value>[];
  value: Value;
  onValueChange: (value: Value) => void;
  disabled?: boolean;
  orientation?: "horizontal" | "vertical";
  hideLabel?: boolean;
  error?: string;
}
export function RadioGroup<Value extends string>({ label, name, options, value, onValueChange, disabled, hideLabel, error, orientation = "vertical", segmented = false }: RadioGroupProps<Value> & { segmented?: boolean }) {
  const errorId = useId();
  return <fieldset className="n-choice-group" role="radiogroup" disabled={disabled} aria-invalid={error ? true : undefined} aria-describedby={error ? errorId : undefined}>
    <legend className={hideLabel ? "n-sr-only" : undefined}>{label}</legend>
    <div className={segmented ? "n-segmented" : cx("n-choices", orientation === "vertical" && "n-vertical")}>
      {options.map(option => <label key={option.value} className={segmented ? "n-segment" : "n-check"}>
        <input type="radio" name={name} value={option.value} checked={value === option.value} disabled={option.disabled}
          onChange={() => onValueChange(option.value)} /><span>{option.label}</span>
      </label>)}
    </div>
    {error ? <span id={errorId} className="n-field-error" role="alert">{error}</span> : null}
  </fieldset>;
}
export function SegmentedControl<Value extends string>(props: RadioGroupProps<Value>) {
  return <RadioGroup {...props} segmented />;
}

export interface CheckboxGroupProps {
  label: string;
  options: readonly ChoiceOption[];
  values: readonly string[];
  onValuesChange: (values: string[]) => void;
  disabled?: boolean;
  orientation?: "horizontal" | "vertical";
}
export function CheckboxGroup({ label, options, values, onValuesChange, disabled, orientation = "horizontal" }: CheckboxGroupProps) {
  return <fieldset className="n-choice-group" disabled={disabled}><legend>{label}</legend>
    <div className={cx("n-choices", orientation === "vertical" && "n-vertical")}>
      {options.map(option => <Checkbox key={option.value} label={option.label} value={option.value} disabled={option.disabled}
        checked={values.includes(option.value)} onChange={event => onValuesChange(event.target.checked ? [...values, option.value] : values.filter(value => value !== option.value))} />)}
    </div>
  </fieldset>;
}
