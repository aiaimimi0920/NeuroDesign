import { useId, useLayoutEffect, useRef, useState, type ReactElement, type ReactNode, type RefObject } from "react";
import * as RadixDialog from "@radix-ui/react-dialog";
import * as AlertDialog from "@radix-ui/react-alert-dialog";
import * as Dropdown from "@radix-ui/react-dropdown-menu";
import * as Tooltip from "@radix-ui/react-tooltip";
import { Button, IconButton } from "./actions";
import { Icon } from "./icons";
import { cx, useNeuroTheme } from "./theme";
import { Tab, TabList, TabPanel, TabsRoot } from "./tabs";

type ReturnFocusResolver = (target: HTMLElement | null) => HTMLElement | null;
function useBusyDialogFocus(busy: boolean | undefined) {
  const content = useRef<HTMLDivElement | null>(null);
  useLayoutEffect(() => {
    const active = document.activeElement;
    // A disabled focused action cannot receive Radix's trapped-focus restoration.
    if (busy && active?.matches(":disabled") && content.current?.contains(active)) content.current.focus();
  }, [busy]);
  return content;
}
function restoreDialogFocus(event: Event, target: HTMLElement | null, resolve?: ReturnFocusResolver) {
  const resolved = resolve ? resolve(target) : target;
  // A background dialog may close after a different modal has already taken focus.
  const activeDialog = document.activeElement?.closest('[role="dialog"], [role="alertdialog"]');
  if (activeDialog?.isConnected && !activeDialog.contains(resolved)) {
    event.preventDefault();
    return;
  }
  if (!resolved?.isConnected || resolved.matches(":disabled") || resolved.closest("[inert]")) return;
  event.preventDefault();
  resolved.focus();
}

/** Extracted from the application tooltip: Radix behavior, Neuro appearance. */
export function ActionTooltip({ label, children }: { label: ReactNode; children: ReactElement }) {
  const { portal } = useNeuroTheme();
  return <Tooltip.Root><Tooltip.Trigger asChild>{children}</Tooltip.Trigger>
    {portal ? <Tooltip.Portal container={portal}><Tooltip.Content className="n-tooltip" collisionPadding={8} side="top" sideOffset={6}>
      {label}<Tooltip.Arrow className="n-tooltip-arrow" />
    </Tooltip.Content></Tooltip.Portal> : null}
  </Tooltip.Root>;
}

export interface DialogProps {
  title: string;
  description?: string;
  trigger?: ReactElement;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  children: ReactNode;
  closeLabel?: string;
  className?: string;
  busy?: boolean;
  dismissOnOutside?: boolean;
  onEscapeKeyDown?: (event: KeyboardEvent) => void;
  initialFocusRef?: RefObject<HTMLElement | null>;
  returnFocusRef?: RefObject<HTMLElement | null>;
  resolveReturnFocus?: ReturnFocusResolver;
  "aria-describedby"?: string;
}
export function Dialog({ title, description, trigger, open, onOpenChange, children, closeLabel = "关闭", className, busy, dismissOnOutside = true, onEscapeKeyDown, initialFocusRef, returnFocusRef, resolveReturnFocus, "aria-describedby": describedBy }: DialogProps) {
  const { portal } = useNeuroTheme();
  const content = useBusyDialogFocus(busy);
  const descriptionId = useId();
  const previousFocus = useRef<HTMLElement | null>(null);
  return <RadixDialog.Root open={open} onOpenChange={next => { if (!busy || next) onOpenChange?.(next); }}>
    {trigger ? <RadixDialog.Trigger asChild>{trigger}</RadixDialog.Trigger> : null}
    {portal ? <RadixDialog.Portal container={portal}>
      <RadixDialog.Overlay className="n-overlay" />
      <RadixDialog.Content ref={content} className={cx("n-dialog", className)} aria-busy={busy || undefined}
        aria-describedby={[description ? descriptionId : undefined, describedBy].filter(Boolean).join(" ") || undefined}
        onEscapeKeyDown={event => { onEscapeKeyDown?.(event); if (busy) event.preventDefault(); }}
        onPointerDownOutside={event => { if (busy || !dismissOnOutside) event.preventDefault(); }}
        onOpenAutoFocus={event => {
          previousFocus.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
          const target = initialFocusRef?.current;
          if (target?.isConnected && !target.matches(":disabled") && !target.closest("[inert]")) {
            event.preventDefault();
            target.focus();
          }
        }}
        onCloseAutoFocus={event => restoreDialogFocus(event, returnFocusRef?.current ?? previousFocus.current, resolveReturnFocus)}>
        <RadixDialog.Title className="n-dialog-title">{title}</RadixDialog.Title>
        {description ? <RadixDialog.Description id={descriptionId} className="n-dialog-description">{description}</RadixDialog.Description> : null}
        <RadixDialog.Close asChild><IconButton className="n-dialog-close" variant="quiet" compact label={closeLabel} disabled={busy}><Icon name="close" /></IconButton></RadixDialog.Close>
        {children}
      </RadixDialog.Content>
    </RadixDialog.Portal> : null}
  </RadixDialog.Root>;
}

export interface ConfirmDialogProps {
  title: string;
  description: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
  confirmLabel?: string;
  cancelLabel?: string;
  busy?: boolean;
  confirmDisabled?: boolean;
  focusResetKey?: string | number;
  dismissOnOutside?: boolean;
  children?: ReactNode;
  danger?: boolean;
  returnFocusRef?: RefObject<HTMLElement | null>;
  resolveReturnFocus?: ReturnFocusResolver;
}
/** The caller closes the dialog after its operation succeeds; failures stay visible. */
export function ConfirmDialog({ title, description, open, onOpenChange, onConfirm, confirmLabel = "确认", cancelLabel = "取消", busy, confirmDisabled, focusResetKey, dismissOnOutside = false, children, danger = true, returnFocusRef, resolveReturnFocus }: ConfirmDialogProps) {
  const { portal } = useNeuroTheme();
  const content = useBusyDialogFocus(busy);
  const previousFocus = useRef<HTMLElement | null>(null);
  const cancelRef = useRef<HTMLButtonElement | null>(null);
  const previousRequest = useRef({ open, key: focusResetKey });
  useLayoutEffect(() => {
    // Initial focus belongs to Radix: it must capture the external return target
    // before focusing Cancel. Only an already-open queue needs an explicit reset.
    if (open && previousRequest.current.open && previousRequest.current.key !== focusResetKey) cancelRef.current?.focus();
    previousRequest.current = { open, key: focusResetKey };
  }, [open, focusResetKey]);
  return <AlertDialog.Root open={open} onOpenChange={onOpenChange}>
    {portal ? <AlertDialog.Portal container={portal}>
      <AlertDialog.Overlay className="n-overlay" onPointerDown={event => {
        if (dismissOnOutside && !busy && event.target === event.currentTarget) onOpenChange(false);
      }} />
      <AlertDialog.Content ref={content} className="n-dialog" onEscapeKeyDown={event => { if (busy) event.preventDefault(); }}
        onOpenAutoFocus={() => { previousFocus.current = document.activeElement instanceof HTMLElement ? document.activeElement : null; }}
        onCloseAutoFocus={event => restoreDialogFocus(event, returnFocusRef?.current ?? previousFocus.current, resolveReturnFocus)}>
        <AlertDialog.Title className="n-dialog-title">{title}</AlertDialog.Title>
        <AlertDialog.Description className="n-dialog-description">{description}</AlertDialog.Description>
        {children}
        <div className="n-dialog-actions">
          <AlertDialog.Cancel asChild><Button ref={cancelRef} disabled={busy}>{cancelLabel}</Button></AlertDialog.Cancel>
          <AlertDialog.Action asChild><Button variant={danger ? "danger" : "primary"} loading={busy} disabled={confirmDisabled}
            onClick={event => { event.preventDefault(); onConfirm(); }}>{confirmLabel}</Button></AlertDialog.Action>
        </div>
      </AlertDialog.Content>
    </AlertDialog.Portal> : null}
  </AlertDialog.Root>;
}

export interface MenuItem { id: string; label: ReactNode; onSelect: () => void; disabled?: boolean; danger?: boolean }
export function DropdownMenu({ trigger, label, items }: { trigger: ReactElement; label: string; items: readonly MenuItem[] }) {
  const { portal } = useNeuroTheme();
  return <Dropdown.Root><Dropdown.Trigger asChild>{trigger}</Dropdown.Trigger>
    {portal ? <Dropdown.Portal container={portal}><Dropdown.Content className="n-menu" sideOffset={6} collisionPadding={12} aria-label={label}>
      {items.map(item => <Dropdown.Item key={item.id} disabled={item.disabled} className="n-menu-item" data-danger={item.danger || undefined} onSelect={item.onSelect}>{item.label}</Dropdown.Item>)}
    </Dropdown.Content></Dropdown.Portal> : null}
  </Dropdown.Root>;
}

export interface TabItem { value: string; label: ReactNode; content: ReactNode; disabled?: boolean }
export function Tabs({ label, items, value, onValueChange }: { label: string; items: readonly TabItem[]; value: string; onValueChange: (value: string) => void }) {
  return <TabsRoot value={value} onValueChange={onValueChange}>
    <TabList label={label}>
      {items.map(item => <Tab key={item.value} value={item.value} disabled={item.disabled}>{item.label}</Tab>)}
    </TabList>
    {items.map(item => <TabPanel key={item.value} value={item.value}>{item.content}</TabPanel>)}
  </TabsRoot>;
}

export interface DisclosureProps {
  title: ReactNode;
  children: ReactNode;
  defaultOpen?: boolean;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  className?: string;
}
export function Disclosure({ title, children, defaultOpen = false, open, onOpenChange, className }: DisclosureProps) {
  const [localOpen, setLocalOpen] = useState(defaultOpen);
  const expanded = open ?? localOpen;
  const contentId = useId();
  return <details className={cx("n-disclosure", className)} open={expanded}>
    <summary aria-expanded={expanded} aria-controls={contentId} onClick={event => {
      // Prevent the native toggle from diverging from an application-owned accordion.
      event.preventDefault();
      if (open === undefined) setLocalOpen(!expanded);
      onOpenChange?.(!expanded);
    }}>{title}<Icon name="chevron" /></summary><div id={contentId}>{children}</div>
  </details>;
}
