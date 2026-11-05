import { useRef, useState } from "react";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { Button, ConfirmDialog, Dialog, Input, NeuroProvider } from "../src/index";

it("lets application capture modified Escape without dismissing or breaking plain Escape", async () => {
  const captured = vi.fn();
  function Example() {
    const [open, setOpen] = useState(false), capture = useRef<HTMLButtonElement>(null);
    return <><Button onClick={() => setOpen(true)}>入口</Button><Dialog title="按键" open={open} onOpenChange={setOpen} initialFocusRef={capture}
      onEscapeKeyDown={event => { if (event.ctrlKey && event.target === capture.current) event.preventDefault(); }}>
      <button ref={capture} onKeyDown={event => { if (event.ctrlKey) { event.preventDefault(); captured(event.key); } }}>捕获</button>
    </Dialog></>;
  }
  render(<NeuroProvider><Example /></NeuroProvider>);
  const user = userEvent.setup(), trigger = screen.getByRole("button", { name: "入口" });
  await user.click(trigger);
  expect(await screen.findByRole("button", { name: "捕获" })).toHaveFocus();
  await user.keyboard("{Control>}{Escape}{/Control}");
  expect(captured).toHaveBeenCalledWith("Escape");
  expect(screen.getByRole("dialog")).toBeVisible();
  await user.keyboard("{Escape}");
  await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  expect(trigger).toHaveFocus();
});

it("keeps busy dismissal protection even when the application receives Escape", async () => {
  const escape = vi.fn(), change = vi.fn();
  render(<NeuroProvider><Dialog title="忙碌" open busy onOpenChange={change} onEscapeKeyDown={escape}>处理中</Dialog></NeuroProvider>);
  await screen.findByRole("dialog");
  await userEvent.setup().keyboard("{Escape}");
  expect(escape).toHaveBeenCalledOnce();
  expect(change).not.toHaveBeenCalled();
});

it.each(["dialog", "confirmation"])("retains a valid focus target when the focused %s action becomes busy", async kind => {
  function Example() {
    const [busy, setBusy] = useState(false);
    return kind === "dialog"
      ? <Dialog open title="保存" busy={busy}><Input aria-label="草稿" /><Button loading={busy} onClick={() => setBusy(true)}>执行</Button></Dialog>
      : <ConfirmDialog open title="确认" description="风险" busy={busy} onOpenChange={() => {}} onConfirm={() => setBusy(true)} />;
  }
  render(<NeuroProvider><Example /></NeuroProvider>);
  const user = userEvent.setup();
  const dialog = await screen.findByRole(kind === "dialog" ? "dialog" : "alertdialog");
  await user.click(screen.getByRole("button", { name: kind === "dialog" ? "执行" : "确认" }));
  expect(dialog).toHaveFocus();
  await user.tab();
  expect(dialog.contains(document.activeElement)).toBe(true);
});
