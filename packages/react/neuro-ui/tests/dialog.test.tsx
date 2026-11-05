import { useRef, useState } from "react";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { Button, Dialog, Input, NeuroProvider } from "../src/index";

it("focuses the requested field and restores its external opening button", async () => {
  function Example() {
    const [open, setOpen] = useState(false);
    const field = useRef<HTMLInputElement | null>(null);
    return <><Button onClick={() => setOpen(true)}>编辑入口</Button>
      <Dialog open={open} onOpenChange={setOpen} title="编辑" description="目标服务" initialFocusRef={field}
        className="application-layout" aria-describedby="validation">
        <Input ref={field} aria-label="名称" /><p id="validation">名称不能为空</p>
      </Dialog></>;
  }
  render(<NeuroProvider mode="light"><Example /></NeuroProvider>);
  const user = userEvent.setup();
  const trigger = screen.getByRole("button", { name: "编辑入口" });
  await user.click(trigger);
  const dialog = await screen.findByRole("dialog");
  expect(screen.getByRole("textbox", { name: "名称" })).toHaveFocus();
  expect(dialog).toHaveClass("n-dialog", "application-layout");
  expect(dialog).toHaveAccessibleDescription("目标服务 名称不能为空");
  expect(dialog.closest("[data-neuro-mode]")).toHaveAttribute("data-neuro-mode", "light");
  await user.keyboard("{Escape}");
  await waitFor(() => expect(trigger).toHaveFocus());
});

it("uses Radix default focus when the requested field is disabled", async () => {
  function Example() {
    const field = useRef<HTMLInputElement | null>(null);
    return <Dialog open title="编辑" initialFocusRef={field}><Input ref={field} disabled aria-label="名称" /></Dialog>;
  }
  render(<NeuroProvider><Example /></NeuroProvider>);
  await screen.findByRole("dialog");
  expect(screen.getByRole("button", { name: "关闭" })).toHaveFocus();
});

it("blocks every dismissal while busy and allows them again after work settles", async () => {
  const change = vi.fn();
  const tree = (busy: boolean) => <NeuroProvider><Dialog open title="编辑" busy={busy} onOpenChange={change}><Input aria-label="名称" /></Dialog></NeuroProvider>;
  const { rerender, container } = render(tree(true));
  const user = userEvent.setup();
  expect(await screen.findByRole("dialog")).toHaveAttribute("aria-busy", "true");
  expect(screen.getByRole("button", { name: "关闭" })).toBeDisabled();
  await user.keyboard("{Escape}");
  await user.click(container.querySelector(".n-overlay")!);
  expect(change).not.toHaveBeenCalled();
  rerender(tree(false));
  await user.click(container.querySelector(".n-overlay")!);
  expect(change).toHaveBeenCalledExactlyOnceWith(false);
});

it("retains an uncontrolled dialog's trigger behavior", async () => {
  render(<NeuroProvider><Dialog title="编辑" trigger={<Button>打开</Button>}><Input aria-label="名称" /></Dialog></NeuroProvider>);
  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "打开" }));
  expect(await screen.findByRole("dialog")).toBeVisible();
  await user.keyboard("{Escape}");
  await waitFor(() => expect(screen.getByRole("button", { name: "打开" })).toHaveFocus());
});
