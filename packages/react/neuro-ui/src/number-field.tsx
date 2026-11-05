import { useEffect, useId, useState } from "react";
import { IconButton } from "./actions";
import { Icon } from "./icons";

export function normalizeNumber(value: number, min: number, max: number, step: number) {
  if (![value, min, max, step].every(Number.isFinite) || step <= 0 || max < min) throw new RangeError("Invalid numeric range or value");
  const clamped = Math.min(max, Math.max(min, value));
  const snapped = min + Math.round((clamped - min) / step) * step;
  return Number(Math.min(max, Math.max(min, snapped)).toPrecision(12));
}
export interface NumberFieldProps {
  label: string;
  value: number | null;
  onValueChange: (value: number | null) => void;
  min: number;
  max: number;
  step: number;
  unit?: string;
  disabled?: boolean;
  allowEmpty?: boolean;
  decreaseLabel?: string;
  increaseLabel?: string;
}
export function NumberField({ label, value, onValueChange, min, max, step, unit, disabled, allowEmpty = false, decreaseLabel = "减少", increaseLabel = "增加" }: NumberFieldProps) {
  const id = useId();
  const [draft, setDraft] = useState(value === null ? "" : String(value));
  useEffect(() => setDraft(value === null ? "" : String(value)), [value]);
  function commit() {
    if (draft.trim() === "" || !Number.isFinite(Number(draft))) {
      if (allowEmpty) onValueChange(null);
      else setDraft(value === null ? "" : String(value));
      return;
    }
    const next = normalizeNumber(Number(draft), min, max, step);
    setDraft(String(next));
    if (next !== value) onValueChange(next);
  }
  function move(direction: -1 | 1) {
    const base = draft.trim() !== "" && Number.isFinite(Number(draft)) ? Number(draft) : value;
    if (base === null) return;
    const next = normalizeNumber(normalizeNumber(base, min, max, step) + direction * step, min, max, step);
    setDraft(String(next));
    onValueChange(next);
  }
  return <div className="n-field"><label htmlFor={id}>{label}</label><div className="n-number-field">
    <IconButton label={`${decreaseLabel} ${label}`} compact variant="quiet" disabled={disabled || value === null || value <= min} onClick={() => move(-1)}><Icon name="minus" /></IconButton>
    <input id={id} type="number" value={draft} min={min} max={max} step={step} disabled={disabled} required={!allowEmpty}
      aria-label={unit ? `${label} (${unit})` : label} onChange={event => setDraft(event.target.value)} onBlur={commit}
      onKeyDown={event => { if (event.key === "Enter") { event.preventDefault(); commit(); } }} />
    {unit ? <span className="n-number-unit" aria-hidden="true">{unit}</span> : null}
    <IconButton label={`${increaseLabel} ${label}`} compact variant="quiet" disabled={disabled || value === null || value >= max} onClick={() => move(1)}><Icon name="plus" /></IconButton>
  </div></div>;
}
