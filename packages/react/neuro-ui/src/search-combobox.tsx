import { useEffect, useId, useRef, useState, type InputHTMLAttributes, type ReactNode } from "react";
import * as Popover from "@radix-ui/react-popover";
import { Input } from "./fields";
import { IconButton } from "./actions";
import { Icon } from "./icons";
import { cx, useNeuroTheme } from "./theme";

export interface SearchOption {
  value: string;
  label: ReactNode;
  description?: ReactNode;
  disabled?: boolean;
}
export interface SearchComboboxProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "value" | "onChange" | "onSelect" | "size"> {
  label: string;
  value: string;
  onValueChange: (value: string) => void;
  options: readonly SearchOption[];
  onSelect: (value: string) => void;
  onExpandedChange?: (expanded: boolean) => void;
  clearLabel?: string;
  emptyLabel?: string;
  compact?: boolean;
}

/** Suggestions retain input focus. Filtering, indexing and navigation targets belong to the caller. */
export function SearchCombobox({ label, value, onValueChange, options, onSelect, onExpandedChange,
  clearLabel = "清空搜索", emptyLabel = "没有匹配结果", compact, disabled, className, ...props }: SearchComboboxProps) {
  const { portal } = useNeuroTheme();
  const id = useId();
  const root = useRef<HTMLDivElement>(null);
  const input = useRef<HTMLInputElement>(null);
  const [open, setOpen] = useState(false);
  const [index, setIndex] = useState(0);
  const expanded = open && !!value.trim() && !disabled;
  const active = options[index]?.disabled ? -1 : Math.min(index, options.length - 1);
  useEffect(() => { onExpandedChange?.(expanded); return () => onExpandedChange?.(false); }, [expanded, onExpandedChange]);
  useEffect(() => { if (disabled) setOpen(false); }, [disabled]);
  useEffect(() => { if (expanded && active >= 0) document.getElementById(`${id}-${active}`)?.scrollIntoView?.({ block: "nearest" }); }, [expanded, active, id]);
  const select = (position: number) => {
    const option = options[position];
    if (!option || option.disabled || disabled) return;
    setOpen(false);
    onSelect(option.value);
  };
  return <Popover.Root open={expanded} onOpenChange={setOpen}>
    <Popover.Anchor asChild><div ref={root} className={cx("n-search-combobox", className)}>
      <Icon name="search" />
      <Input {...props} ref={input} compact={compact} role="combobox" aria-label={label} aria-autocomplete="list"
        aria-expanded={expanded} aria-controls={expanded ? id : undefined} disabled={disabled}
        aria-activedescendant={expanded && active >= 0 ? `${id}-${active}` : undefined} value={value}
        onFocus={event => { setOpen(true); props.onFocus?.(event); }}
        onBlur={event => { setOpen(false); props.onBlur?.(event); }}
        onChange={event => { onValueChange(event.target.value); setIndex(0); setOpen(true); }}
        onKeyDown={event => {
          props.onKeyDown?.(event);
          if (event.defaultPrevented) return;
          if (event.key === "Escape" && (expanded || value)) {
            event.preventDefault(); event.stopPropagation(); setOpen(false); onValueChange("");
          } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
            event.preventDefault(); setOpen(true);
            const enabled = options.map((option, position) => option.disabled ? -1 : position).filter(position => position >= 0);
            const current = enabled.indexOf(active);
            const step = event.key === "ArrowDown" ? 1 : -1;
            setIndex(enabled.length ? enabled[(current + step + enabled.length) % enabled.length] : 0);
          } else if (event.key === "Enter" && expanded) { event.preventDefault(); select(active); }
        }} />
      {value ? <IconButton compact variant="quiet" label={clearLabel} disabled={disabled}
        onClick={() => { onValueChange(""); setOpen(false); input.current?.focus(); }}><Icon name="close" /></IconButton> : null}
    </div></Popover.Anchor>
    {portal ? <Popover.Portal container={portal}>
      <Popover.Content className="n-search-suggestions" role="presentation" align="end" sideOffset={4} collisionPadding={8}
        onOpenAutoFocus={event => event.preventDefault()} onCloseAutoFocus={event => event.preventDefault()}
        onInteractOutside={event => { if (root.current?.contains(event.target as Node)) event.preventDefault(); }}
        onEscapeKeyDown={event => { event.preventDefault(); event.stopPropagation(); setOpen(false); onValueChange(""); }}>
        <div id={id} role="listbox" aria-label={label}>
          {options.map((option, position) => <div key={option.value} role="option" id={`${id}-${position}`}
            aria-label={typeof option.label === "string" ? option.label : undefined}
            aria-selected={position === active} aria-disabled={option.disabled || undefined} className="n-search-option"
            onMouseDown={event => event.preventDefault()} onMouseMove={() => { if (!option.disabled) setIndex(position); }}
            onClick={() => select(position)}>
            <span>{option.label}</span>{option.description ? <small>{option.description}</small> : null}
          </div>)}
          {!options.length ? <div className="n-search-empty" role="status">{emptyLabel}</div> : null}
        </div>
      </Popover.Content>
    </Popover.Portal> : null}
  </Popover.Root>;
}
