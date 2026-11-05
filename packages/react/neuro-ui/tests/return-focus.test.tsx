import { useRef, useState } from "react";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { Button, ConfirmDialog, Dialog, NeuroProvider } from "../src/index";

it.each(["dialog", "confirmation"])("resolves the %s return target after application state disables its opener", async kind => {
  const resolve = vi.fn();
  function Example() {
    const [open, setOpen] = useState(false);
    const [disabled, setDisabled] = useState(false);
    const fallback = useRef<HTMLButtonElement | null>(null);
    const returnTarget = (target: HTMLElement | null) => {
      resolve(target);
      return target?.matches(":disabled") ? fallback.current : target;
    };
    const finish = () => { setDisabled(true); setOpen(false); };
    return <><Button disabled={disabled} onClick={() => setOpen(true)}>原入口</Button><Button ref={fallback}>持续入口</Button>
      {kind === "dialog" ? <Dialog open={open} title="编辑" onOpenChange={setOpen} resolveReturnFocus={returnTarget}>
        <Button onClick={finish}>保存</Button>
      </Dialog> : <ConfirmDialog open={open} title="确认" description="操作后原入口不可用" onOpenChange={setOpen}
        resolveReturnFocus={returnTarget} onConfirm={finish} />}</>;
  }
  render(<NeuroProvider><Example /></NeuroProvider>);
  const user = userEvent.setup();
  const opener = screen.getByRole("button", { name: "原入口" });
  await user.click(opener);
  await user.click(screen.getByRole("button", { name: kind === "dialog" ? "保存" : "确认" }));
  await waitFor(() => expect(screen.getByRole("button", { name: "持续入口" })).toHaveFocus());
  expect(resolve).toHaveBeenLastCalledWith(opener);
  expect(opener).toBeDisabled();
});

it.each(["dialog", "confirmation"])("does not restore a closing %s over a different active dialog", async kind => {
  let openOther = () => {};
  let closeFirst = () => {};
  function Example() {
    const [first, setFirst] = useState(false);
    const [other, setOther] = useState(false);
    openOther = () => setOther(true);
    closeFirst = () => setFirst(false);
    return <><Button onClick={() => setFirst(true)}>原入口</Button>
      {kind === "dialog" ? <Dialog open={first} onOpenChange={setFirst} title="原弹窗">内容</Dialog>
        : <ConfirmDialog open={first} onOpenChange={setFirst} title="原确认" description="确认内容" onConfirm={() => {}} />}
      <Dialog open={other} onOpenChange={setOther} title="其他弹窗"><Button>其他操作</Button></Dialog>
    </>;
  }
  render(<NeuroProvider><Example /></NeuroProvider>);
  const user = userEvent.setup();
  const opener = screen.getByRole("button", { name: "原入口" });
  await user.click(opener);
  act(openOther);
  const other = await screen.findByRole("dialog", { name: "其他弹窗" });
  const close = within(other).getByRole("button", { name: "关闭" });
  await waitFor(() => expect(close).toHaveFocus());
  const focus = vi.spyOn(opener, "focus");
  act(closeFirst);
  await waitFor(() => expect(document.querySelectorAll('[role="dialog"], [role="alertdialog"]')).toHaveLength(1));
  await new Promise(resolve => setTimeout(resolve, 20));
  expect(close).toHaveFocus();
  expect(focus).not.toHaveBeenCalled();
});

it.each(["disabled", "inert", "detached", "null"])("does not intercept Radix trigger restoration for a %s resolved target", async state => {
  function Example() {
    const candidate = useRef<HTMLButtonElement | null>(null);
    const detached = document.createElement("button");
    return <><Button ref={candidate} disabled={state === "disabled"} inert={state === "inert"}>不可用目标</Button>
      <Dialog title="编辑" trigger={<Button>原入口</Button>} resolveReturnFocus={() => state === "null" ? null : state === "detached" ? detached : candidate.current}>
        <p>编辑内容</p>
      </Dialog></>;
  }
  render(<NeuroProvider><Example /></NeuroProvider>);
  const user = userEvent.setup();
  const opener = screen.getByRole("button", { name: "原入口" });
  await user.click(opener);
  await user.keyboard("{Escape}");
  await waitFor(() => expect(opener).toHaveFocus());
});
