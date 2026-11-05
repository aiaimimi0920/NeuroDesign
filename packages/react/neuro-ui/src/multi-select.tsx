import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import * as Popover from "@radix-ui/react-popover";
import { Button } from "./actions";
import { Input } from "./fields";
import { Icon } from "./icons";
import { Checkbox, type ChoiceOption, type CheckboxGroupProps } from "./selection";
import { cx, useNeuroTheme } from "./theme";

export interface MultiSelectOption extends ChoiceOption {
  description?: ReactNode;
  meta?: ReactNode;
  searchText?: string;
}
export interface MultiSelectProps extends Omit<CheckboxGroupProps, "orientation" | "options"> {
  options: readonly MultiSelectOption[];
  formatSummary?: (count: number) => string;
  compact?: boolean;
  className?: string;
  search?: { label: string; placeholder?: string; emptyMessage: string };
  empty?: ReactNode;
}

/** Selection stays controlled; applications decide availability and selection limits. */
export function MultiSelect({ label, options, values, onValuesChange, disabled, compact, className, search, empty,
  formatSummary = count => `已选 ${count} 项` }: MultiSelectProps) {
  const { portal } = useNeuroTheme();
  const labelId = useId();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const content = useRef<HTMLDivElement | null>(null);
  const trigger = useRef<HTMLButtonElement | null>(null);
  const openingKey = useRef<"ArrowDown" | "ArrowUp" | null>(null);
  const tabExit = useRef(false);
  const summary = formatSummary(values.length);
  const needle = query.trim().toLocaleLowerCase();
  const visible = search ? options.filter(option =>
    (option.searchText ?? (typeof option.label === "string" ? option.label : option.value)).toLocaleLowerCase().includes(needle)) : options;
  useEffect(() => { if (disabled) { setOpen(false); setQuery(""); } }, [disabled]);
  function changeOpen(next: boolean) {
    setOpen(next);
    if (!next) setQuery("");
  }
  return <div className={cx("n-field n-multi-select", className)}><span id={labelId} className="n-field-label">{label}</span>
    <Popover.Root open={open && !disabled} onOpenChange={changeOpen}><Popover.Trigger asChild>
      <Button ref={trigger} className="n-select-trigger" disabled={disabled} compact={compact} aria-label={`${label}，${summary}`} title={summary}
        onKeyDown={event => {
          if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
          event.preventDefault();
          openingKey.current = event.key;
          if (!open) changeOpen(true);
          else content.current?.querySelector<HTMLInputElement>('input:not(:disabled)')?.focus();
        }}>
        <span>{summary}</span><Icon name="chevron" />
      </Button>
    </Popover.Trigger>
      {portal ? <Popover.Portal container={portal}><Popover.Content ref={content} className="n-popover n-multi-options" sideOffset={6} collisionPadding={12} aria-labelledby={labelId}
        onOpenAutoFocus={event => {
          const selector = search ? 'input[type="search"]' : 'input[type="checkbox"]:not(:disabled)';
          const targets = content.current?.querySelectorAll<HTMLInputElement>(selector);
          const target = openingKey.current === "ArrowUp" && !search ? targets?.[targets.length - 1] : targets?.[0];
          openingKey.current = null;
          if (target) { event.preventDefault(); target.focus(); }
        }}
        onEscapeKeyDown={event => event.stopPropagation()}
        onCloseAutoFocus={event => {
          if (tabExit.current) { event.preventDefault(); tabExit.current = false; }
        }}
        onFocusOutside={event => { if (!trigger.current?.contains(event.target as Node)) changeOpen(false); }}
        onKeyDown={event => {
          if (event.key === "Escape") event.stopPropagation();
          if (event.key === "Tab") {
            const targets = content.current?.querySelectorAll<HTMLInputElement>('input:not(:disabled)');
            const boundary = event.shiftKey ? targets?.[0] : targets?.[targets.length - 1];
            if (event.target === boundary) {
              // Return to the trigger before native Tab advances to the next form field.
              // Radix's default popup loop must not trap users in a non-modal selector.
              tabExit.current = true;
              changeOpen(false);
              trigger.current?.focus();
            }
            return;
          }
          if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
          if (event.target instanceof HTMLInputElement && event.target.type === "search" && !event.key.startsWith("Arrow")) return;
          const targets = Array.from(content.current?.querySelectorAll<HTMLInputElement>('input[type="checkbox"]:not(:disabled)') ?? []);
          if (!targets.length) return;
          event.preventDefault();
          const index = targets.indexOf(document.activeElement as HTMLInputElement);
          const next = event.key === "Home" ? 0 : event.key === "End" ? targets.length - 1 : index < 0
            ? (event.key === "ArrowUp" ? targets.length - 1 : 0) : (index + (event.key === "ArrowUp" ? -1 : 1) + targets.length) % targets.length;
          targets[next].focus();
        }}>
        {search ? <Input type="search" compact aria-label={search.label} placeholder={search.placeholder ?? search.label} value={query} maxLength={256} onChange={event => setQuery(event.target.value)} /> : null}
        <div className="n-multi-list" role="group" aria-labelledby={labelId}>
          {visible.map(option => <Checkbox key={option.value} value={option.value} disabled={option.disabled}
            checked={values.includes(option.value)} label={<><span className="n-multi-option-label"><strong>{option.label}</strong>{option.description ? <small>{option.description}</small> : null}</span>{option.meta ? <small className="n-multi-option-meta">{option.meta}</small> : null}</>}
            onChange={event => onValuesChange(event.target.checked ? [...values, option.value] : values.filter(value => value !== option.value))} />)}
          {!visible.length ? <div className="n-multi-empty">{options.length ? <span role="status">{search?.emptyMessage}</span> : empty ?? <span role="status">暂无选项</span>}</div> : null}
        </div>
      </Popover.Content></Popover.Portal> : null}
    </Popover.Root>
  </div>;
}
