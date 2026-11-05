import { useRef, useState } from "react";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Button, Checkbox, ConfirmDialog, NeuroProvider, PopupSelect, ToastViewport } from "../src/index";

describe("application adoption contracts", () => {
  it("confirmation without a Radix trigger restores the original external button", async () => {
    function Example() {
      const [open, setOpen] = useState(false);
      return <><Button onClick={() => setOpen(true)}>删除入口</Button>
        <ConfirmDialog open={open} title="移除" description="风险" onOpenChange={setOpen} onConfirm={() => {}} /></>;
    }
    render(<NeuroProvider><Example /></NeuroProvider>);
    await userEvent.click(screen.getByRole("button", { name: "删除入口" }));
    expect(await screen.findByRole("alertdialog")).toBeVisible();
    await userEvent.keyboard("{Escape}");
    await waitFor(() => expect(screen.getByRole("button", { name: "删除入口" })).toHaveFocus());
  });
  it("popup selects preserve the trigger, theme, disabled options and keyboard return", async () => {
    function Example() {
      const [value, setValue] = useState("a");
      return <PopupSelect aria-label="排序" value={value} onValueChange={setValue} data-owner="fixture"
        options={[{ value: "a", label: "名称" }, { value: "b", label: "时间" }, { value: "c", label: "锁定", disabled: true }]} />;
    }
    render(<NeuroProvider mode="light"><Example /></NeuroProvider>);
    const user = userEvent.setup();
    const trigger = screen.getByRole("combobox", { name: "排序" });
    await user.click(trigger);
    expect(screen.getByRole("listbox").closest("[data-neuro-mode]")).toHaveAttribute("data-neuro-mode", "light");
    expect(screen.getByRole("option", { name: "锁定" })).toHaveAttribute("aria-disabled", "true");
    await user.click(screen.getByRole("option", { name: "时间" }));
    expect(screen.getByRole("combobox")).toBe(trigger);
    expect(trigger).toHaveTextContent("时间");
    expect(trigger).toHaveAttribute("data-owner", "fixture");
    await user.click(trigger);
    await user.keyboard("{Escape}");
    await waitFor(() => expect(trigger).toHaveFocus());
  });

  it("disabled confirmation remains cancellable and keeps optional content", async () => {
    const cancel = vi.fn();
    const confirm = vi.fn();
    render(<NeuroProvider><ConfirmDialog open title="移除" description="风险说明" confirmDisabled onOpenChange={cancel} onConfirm={confirm}>
      <Checkbox label="同时清理依赖" defaultChecked />
    </ConfirmDialog></NeuroProvider>);
    expect(await screen.findByRole("checkbox")).toBeChecked();
    expect(screen.getByRole("button", { name: "确认" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "取消" })).toBeEnabled();
    await userEvent.click(screen.getByRole("button", { name: "取消" }));
    expect(cancel).toHaveBeenCalledWith(false);
    expect(confirm).not.toHaveBeenCalled();
  });

  it("queue identity resets focus without replacing an open dialog", async () => {
    function Example() {
      const [id, setId] = useState(1);
      const [open, setOpen] = useState(false);
      const trigger = useRef<HTMLButtonElement | null>(null);
      return <><Button ref={trigger} onClick={() => setOpen(true)}>开始</Button>
        <ConfirmDialog open={open} title={`请求 ${id}`} description="确认请求" focusResetKey={id} returnFocusRef={trigger}
          onOpenChange={setOpen} onConfirm={() => setId(2)} />
      </>;
    }
    render(<NeuroProvider><Example /></NeuroProvider>);
    await userEvent.click(screen.getByRole("button", { name: "开始" }));
    const dialog = await screen.findByRole("alertdialog");
    await userEvent.click(screen.getByRole("button", { name: "确认" }));
    expect(screen.getByRole("alertdialog")).toBe(dialog);
    expect(dialog).toHaveAccessibleName("请求 2");
    expect(screen.getByRole("button", { name: "取消" })).toHaveFocus();
    await userEvent.keyboard("{Escape}");
    await waitFor(() => expect(screen.getByRole("button", { name: "开始" })).toHaveFocus());
  });

  it("outside cancellation is opt-in and blocked while busy", () => {
    const change = vi.fn();
    const { container, rerender } = render(<NeuroProvider><ConfirmDialog open title="请求" description="风险" onOpenChange={change} onConfirm={() => {}} /></NeuroProvider>);
    fireEvent.pointerDown(container.querySelector(".n-overlay")!);
    expect(change).not.toHaveBeenCalled();
    rerender(<NeuroProvider><ConfirmDialog open title="请求" description="风险" dismissOnOutside busy onOpenChange={change} onConfirm={() => {}} /></NeuroProvider>);
    fireEvent.pointerDown(container.querySelector(".n-overlay")!);
    expect(change).not.toHaveBeenCalled();
    rerender(<NeuroProvider><ConfirmDialog open title="请求" description="风险" dismissOnOutside onOpenChange={change} onConfirm={() => {}} /></NeuroProvider>);
    fireEvent.pointerDown(container.querySelector(".n-overlay")!);
    expect(change).toHaveBeenCalledExactlyOnceWith(false);
  });

  it("controlled notifications keep tone lifetimes, one dismissal and release exit timers", () => {
    vi.useFakeTimers();
    const remove = vi.fn();
    const { unmount } = render(<NeuroProvider><ToastViewport entries={[
      { id: 1, message: "信息", tone: "info" }, { id: 2, message: "警告", tone: "warning" }, { id: 3, message: "错误", tone: "danger" },
    ]} onRemove={remove} /></NeuroProvider>);
    expect(screen.getByRole("alert")).toHaveAttribute("aria-live", "assertive");
    act(() => vi.advanceTimersByTime(3420));
    expect(remove).toHaveBeenCalledExactlyOnceWith(1);
    fireEvent.click(screen.getByRole("status", { name: /警告/ }));
    fireEvent.click(screen.getByRole("status", { name: /警告/ }));
    act(() => vi.advanceTimersByTime(220));
    expect(remove.mock.calls).toEqual([[1], [2]]);
    fireEvent.click(screen.getByRole("alert"));
    unmount();
    expect(vi.getTimerCount()).toBe(0);
  });
});
