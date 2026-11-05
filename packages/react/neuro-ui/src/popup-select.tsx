import type { ComponentPropsWithoutRef, ReactNode, Ref } from "react";
import * as Select from "@radix-ui/react-select";
import { Icon } from "./icons";
import { cx, useNeuroTheme } from "./theme";

export interface PopupSelectOption<Value extends string = string> {
  value: Value;
  label: ReactNode;
  textValue?: string;
  disabled?: boolean;
}
export interface PopupSelectProps<Value extends string = string> extends Omit<ComponentPropsWithoutRef<typeof Select.Trigger>, "children" | "value" | "defaultValue" | "onChange"> {
  value: Value;
  onValueChange: (value: Value) => void;
  options: readonly PopupSelectOption<Value>[];
  placeholder?: string;
  leading?: ReactNode;
  displayValue?: ReactNode;
  compact?: boolean;
  name?: string;
  required?: boolean;
  ref?: Ref<HTMLButtonElement>;
}

/** A themed popup, with Radix keyboard navigation and nonempty option values. */
export function PopupSelect<Value extends string>({ value, onValueChange, options, placeholder, leading, displayValue, compact, name, required, className, disabled, ref, ...props }: PopupSelectProps<Value>) {
  const { portal } = useNeuroTheme();
  return <Select.Root value={value} onValueChange={onValueChange} disabled={disabled} name={name} required={required}>
    <Select.Trigger {...props} ref={ref} className={cx("n-button", "n-popup-select-trigger", compact && "n-compact", className)}>
      {leading}<Select.Value className="n-popup-select-value" placeholder={placeholder}>{displayValue}</Select.Value><Select.Icon><Icon name="chevron" /></Select.Icon>
    </Select.Trigger>
    {portal ? <Select.Portal container={portal}>
      <Select.Content className="n-popup-select" position="popper" align="start" sideOffset={4} collisionPadding={8}>
        <Select.ScrollUpButton className="n-popup-select-scroll"><Icon name="chevron" /></Select.ScrollUpButton>
        <Select.Viewport className="n-popup-select-options">
          {options.map(option => <Select.Item key={option.value} value={option.value} disabled={option.disabled} textValue={option.textValue ?? (typeof option.label === "string" ? option.label : undefined)} className="n-popup-select-option">
            <Select.ItemText>{option.label}</Select.ItemText><Select.ItemIndicator><Icon name="check" /></Select.ItemIndicator>
          </Select.Item>)}
        </Select.Viewport>
        <Select.ScrollDownButton className="n-popup-select-scroll"><Icon name="chevron" /></Select.ScrollDownButton>
      </Select.Content>
    </Select.Portal> : null}
  </Select.Root>;
}
