import { Fragment, useLayoutEffect, useRef, type ReactNode } from "react";
import { Checkbox } from "./selection";

export interface TableColumn<T> {
  key: string;
  header: ReactNode;
  cell: (row: T) => ReactNode;
  width?: string;
  rowHeader?: boolean;
}
export interface DataTableProps<T> {
  label: string;
  rows: readonly T[];
  columns: readonly TableColumn<T>[];
  getRowKey: (row: T) => string;
  selectedKeys?: readonly string[];
  onSelectedKeysChange?: (keys: string[]) => void;
  getRowLabel?: (row: T) => string;
  selectAllLabel?: string;
  activeKey?: string;
  empty?: ReactNode;
  rowDetails?: (row: T) => { id: string; open: boolean; content: ReactNode };
}
export function DataTable<T>({ label, rows, columns, getRowKey, selectedKeys = [], onSelectedKeysChange, getRowLabel = getRowKey,
  selectAllLabel = "选择当前列表全部项目", activeKey, empty, rowDetails }: DataTableProps<T>) {
  const keys = rows.map(getRowKey);
  const count = keys.filter(key => selectedKeys.includes(key)).length;
  return <div className="n-table-scroll" role="region" aria-label={label} tabIndex={0}>
    <table className="n-table" aria-label={label}><colgroup>{onSelectedKeysChange ? <col style={{ width: 40 }} /> : null}{columns.map(column => <col key={column.key} style={{ width: column.width }} />)}</colgroup>
      <thead><tr>{onSelectedKeysChange ? <th scope="col"><Checkbox aria-label={selectAllLabel} disabled={!rows.length}
        checked={rows.length > 0 && count === rows.length} indeterminate={count > 0 && count < rows.length}
        onChange={event => onSelectedKeysChange(event.target.checked ? [...new Set([...selectedKeys, ...keys])] : selectedKeys.filter(key => !keys.includes(key)))} /></th> : null}
        {columns.map(column => <th key={column.key} scope="col">{column.header}</th>)}
      </tr></thead>
      <tbody>{rows.map(row => {
        const key = getRowKey(row);
        const checked = selectedKeys.includes(key);
        const details = rowDetails?.(row);
        return <Fragment key={key}><tr data-row-key={key} data-selected={checked || key === activeKey || undefined}>
          {onSelectedKeysChange ? <td><Checkbox aria-label={`选择 ${getRowLabel(row)}`} checked={checked}
            onChange={event => onSelectedKeysChange(event.target.checked ? [...selectedKeys, key] : selectedKeys.filter(value => value !== key))} /></td> : null}
          {columns.map(column => column.rowHeader ? <th key={column.key} scope="row">{column.cell(row)}</th> : <td key={column.key}>{column.cell(row)}</td>)}
        </tr>{details ? <tr id={details.id} hidden={!details.open} data-row-details={key}><td colSpan={columns.length + (onSelectedKeysChange ? 1 : 0)}>{details.content}</td></tr> : null}</Fragment>;
      })}</tbody>
    </table>
    {!rows.length ? empty : null}
  </div>;
}

/** Both layers reserve the same area; selection never inserts a new toolbar row. */
export function SelectionBar({ count, summary, children, formatCount = value => `已选 ${value} 项` }: {
  count: number; summary: ReactNode; children: ReactNode; formatCount?: (count: number) => string;
}) {
  const selected = count > 0;
  const summaryRef = useRef<HTMLDivElement | null>(null);
  const actionsRef = useRef<HTMLDivElement | null>(null);
  useLayoutEffect(() => {
    if (!selected && actionsRef.current?.contains(document.activeElement)) summaryRef.current?.focus();
  }, [selected]);
  return <div className="n-selection-bar" data-has-selection={selected}>
    <div className="n-selection-summary" ref={summaryRef} tabIndex={-1} inert={selected}>{summary}</div>
    <div className="n-selection-actions" ref={actionsRef} inert={!selected}><span role="status">{formatCount(count)}</span><div className="n-row">{children}</div></div>
  </div>;
}
