import { useRef, useState, type ReactNode } from "react";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import {
  ActionTooltip, Button, Checkbox, CheckboxGroup, ConfirmDialog, CopyValue, DataTable, Dialog, DropdownMenu,
  Field, Icon, IconButton, Input, MultiSelect, NeuroProvider, NumberField, SegmentedControl, SelectionBar,
  Switch, Tabs, ToastProvider, normalizeNumber, useToast,
} from "../src/index";

function wrap(children: ReactNode) { return <NeuroProvider>{children}</NeuroProvider>; }
const options = [{ value: "a", label: "选项 A" }, { value: "b", label: "选项 B" }, { value: "c", label: "不可用", disabled: true }];

describe("actions and fields", () => {
  it("buttons do not accidentally submit forms and block loading actions", async () => {
    const click = vi.fn();
    const { rerender } = render(wrap(<Button onClick={click}>执行</Button>));
    expect(screen.getByRole("button")).toHaveAttribute("type", "button");
    await userEvent.click(screen.getByRole("button"));
    expect(click).toHaveBeenCalledTimes(1);
    rerender(wrap(<Button onClick={click} loading>执行</Button>));
    expect(screen.getByRole("button")).toHaveAttribute("aria-busy", "true");
    expect(screen.getByRole("button")).toBeDisabled();
    await userEvent.click(screen.getByRole("button"));
    expect(click).toHaveBeenCalledTimes(1);
  });
  it("fields associate labels and actual error messages without permanent helper rows", () => {
    const { rerender } = render(wrap(<Field label="名称"><Input /></Field>));
    expect(screen.getByLabelText("名称")).not.toHaveAttribute("aria-describedby");
    rerender(wrap(<Field label="名称" error="名称不能为空"><Input /></Field>));
    expect(screen.getByLabelText("名称")).toHaveAttribute("aria-invalid", "true");
    expect(screen.getByLabelText("名称")).toHaveAccessibleDescription("名称不能为空");
  });
  it("icon buttons expose their accessible names", () => {
    render(wrap(<IconButton label="收藏"><Icon name="star" /></IconButton>));
    expect(screen.getByRole("button", { name: "收藏" })).toBeVisible();
  });
  it("checkbox mixed state is applied to the native element", () => {
    render(wrap(<Checkbox aria-label="全选" indeterminate />));
    expect(screen.getByRole("checkbox")).toBePartiallyChecked();
  });
  it("switches retain native checkbox operation", async () => {
    render(wrap(<Switch label="元数据" defaultChecked />));
    await userEvent.click(screen.getByRole("switch"));
    expect(screen.getByRole("switch")).not.toBeChecked();
  });
});

describe("selection", () => {
  it("segmented controls are one radio group, not tabs or toggles", async () => {
    function Example() { const [value, setValue] = useState("a"); return <SegmentedControl label="模式" name="mode" options={options} value={value} onValueChange={setValue} />; }
    render(wrap(<Example />));
    const original = screen.getByRole("radio", { name: "选项 B" });
    await userEvent.click(original);
    expect(original).toBeChecked();
    expect(screen.getByRole("radio", { name: "选项 A" })).not.toBeChecked();
    expect(screen.getByRole("radio", { name: "选项 B" })).toBe(original);
    expect(screen.queryByRole("tablist")).not.toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "不可用" })).toBeDisabled();
  });
  it("checkbox groups allow multiple values and no selection", async () => {
    function Example() { const [values, setValues] = useState(["a", "b"]); return <CheckboxGroup label="能力" options={options} values={values} onValuesChange={setValues} />; }
    render(wrap(<Example />));
    await userEvent.click(screen.getByRole("checkbox", { name: "选项 A" }));
    await userEvent.click(screen.getByRole("checkbox", { name: "选项 B" }));
    expect(screen.getAllByRole("checkbox").every(input => !(input as HTMLInputElement).checked)).toBe(true);
  });
  it("multi-select stays open when checked and restores trigger focus on Escape", async () => {
    function Example() { const [values, setValues] = useState<string[]>([]); return <MultiSelect label="能力" options={options} values={values} onValuesChange={setValues} />; }
    render(wrap(<Example />));
    const user = userEvent.setup();
    const trigger = screen.getByRole("button", { name: /能力/ });
    await user.click(trigger);
    const original = await screen.findByRole("checkbox", { name: "选项 A" });
    await user.click(original);
    expect(original).toBeChecked();
    expect(screen.getByRole("checkbox", { name: "选项 A" })).toBe(original);
    expect(trigger).toHaveAccessibleName("能力，已选 1 项");
    await user.keyboard("{Escape}");
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(trigger).toHaveFocus();
  });
  it("table selection updates the same rows and preserves focus", async () => {
    const rows = [{ id: "a" }, { id: "b" }];
    function Example() { const [keys, setKeys] = useState<string[]>([]); return <DataTable label="列表" rows={rows} getRowKey={row => row.id} selectedKeys={keys} onSelectedKeysChange={setKeys} columns={[{ key: "id", header: "ID", cell: row => row.id }]} />; }
    render(wrap(<Example />));
    expect(screen.getByRole("table", { name: "列表" })).toBeInTheDocument();
    const checkbox = screen.getByRole("checkbox", { name: "选择 a" });
    const row = checkbox.closest("tr");
    await userEvent.click(checkbox);
    expect(screen.getByRole("checkbox", { name: "选择 a" })).toBe(checkbox);
    expect(checkbox).toHaveFocus();
    expect(checkbox.closest("tr")).toBe(row);
    expect(row).toHaveAttribute("data-selected", "true");
    expect(screen.getByRole("checkbox", { name: "选择当前列表全部项目" })).toBePartiallyChecked();
  });
  it("select-all preserves selections outside the current list", async () => {
    const changed = vi.fn();
    render(wrap(<DataTable label="列表" rows={[{ id: "a" }]} getRowKey={row => row.id} selectedKeys={["hidden"]} onSelectedKeysChange={changed} columns={[{ key: "id", header: "ID", cell: row => row.id }]} />));
    await userEvent.click(screen.getByRole("checkbox", { name: "选择当前列表全部项目" }));
    expect(changed).toHaveBeenCalledWith(["hidden", "a"]);
  });
  it("selection toolbar reserves both layers and makes inactive controls inert", () => {
    const { rerender, container } = render(wrap(<SelectionBar count={0} summary="全部"><Button>清除</Button></SelectionBar>));
    const summary = container.querySelector(".n-selection-summary");
    const actions = container.querySelector(".n-selection-actions");
    expect(actions).toHaveAttribute("inert");
    rerender(wrap(<SelectionBar count={1} summary="全部"><Button>清除</Button></SelectionBar>));
    expect(container.querySelector(".n-selection-summary")).toBe(summary);
    expect(container.querySelector(".n-selection-actions")).toBe(actions);
    expect(summary).toHaveAttribute("inert");
    expect(actions).not.toHaveAttribute("inert");
  });
  it("clearing selection moves focus out of the now-inert action area", async () => {
    function Example() { const [count, setCount] = useState(1); return <SelectionBar count={count} summary="全部资源"><Button onClick={() => setCount(0)}>清除</Button></SelectionBar>; }
    const { container } = render(wrap(<Example />));
    await userEvent.click(screen.getByRole("button", { name: "清除" }));
    expect(container.querySelector(".n-selection-summary")).toHaveFocus();
    expect(container.querySelector(".n-selection-actions")).toHaveAttribute("inert");
  });
});

describe("overlays", () => {
  it("dialogs trap focus, lock scrolling, close with Escape and restore focus", async () => {
    render(wrap(<Dialog title="项目" description="项目设置" trigger={<Button>打开</Button>}><Input aria-label="名称" /><Button>保存</Button></Dialog>));
    const user = userEvent.setup();
    const trigger = screen.getByRole("button", { name: "打开" });
    await user.click(trigger);
    const dialog = await screen.findByRole("dialog");
    expect(dialog).toHaveAccessibleName("项目");
    expect(dialog).toHaveAccessibleDescription("项目设置");
    expect(document.body).toHaveAttribute("data-scroll-locked");
    expect(dialog.contains(document.activeElement)).toBe(true);
    await user.tab({ shift: true });
    expect(dialog.contains(document.activeElement)).toBe(true);
    await user.keyboard("{Escape}");
    await waitFor(() => expect(trigger).toHaveFocus());
    expect(document.body).not.toHaveAttribute("data-scroll-locked");
  });
  it("controlled dialogs without a Trigger also restore the opening element", async () => {
    function Example() { const [open, setOpen] = useState(false); return <><Button onClick={() => setOpen(true)}>打开</Button><Dialog title="编辑" open={open} onOpenChange={setOpen}><Input aria-label="名称" /></Dialog></>; }
    render(wrap(<Example />));
    await userEvent.click(screen.getByRole("button", { name: "打开" }));
    await userEvent.keyboard("{Escape}");
    await waitFor(() => expect(screen.getByRole("button", { name: "打开" })).toHaveFocus());
  });
  it("controlled dialogs can return to a persistent action when their opening control unmounts", async () => {
    function Example() {
      const [open, setOpen] = useState(false);
      const destination = useRef<HTMLButtonElement | null>(null);
      return <><Button ref={destination}>下一步</Button>{!open ? <Button onClick={() => setOpen(true)}>临时入口</Button> : null}
        <Dialog title="编辑" open={open} onOpenChange={setOpen} returnFocusRef={destination}><Input aria-label="名称" /></Dialog></>;
    }
    render(wrap(<Example />));
    await userEvent.click(screen.getByRole("button", { name: "临时入口" }));
    await screen.findByRole("dialog");
    await userEvent.keyboard("{Escape}");
    await waitFor(() => expect(screen.getByRole("button", { name: "下一步" })).toHaveFocus());
  });
  it("confirmation does not report completion or close before the caller does", async () => {
    const confirm = vi.fn();
    render(wrap(<ConfirmDialog title="删除？" description="无法撤销" open onOpenChange={() => {}} onConfirm={confirm} />));
    expect(await screen.findByRole("alertdialog")).toHaveAccessibleDescription("无法撤销");
    expect(screen.getByRole("button", { name: "取消" })).toHaveFocus();
    await userEvent.click(screen.getByRole("button", { name: "确认" }));
    expect(confirm).toHaveBeenCalledOnce();
    expect(screen.getByRole("alertdialog")).toBeVisible();
  });
  it("menu-launched confirmation returns focus to an explicit persistent trigger", async () => {
    function Example() {
      const [open, setOpen] = useState(false);
      const trigger = useRef<HTMLButtonElement | null>(null);
      return <><DropdownMenu label="操作" trigger={<Button ref={trigger}>菜单</Button>} items={[{ id: "delete", label: "删除", onSelect: () => setOpen(true) }]} />
        <ConfirmDialog title="删除？" description="无法撤销" open={open} onOpenChange={setOpen} onConfirm={() => setOpen(false)} returnFocusRef={trigger} /></>;
    }
    render(wrap(<Example />));
    await userEvent.click(screen.getByRole("button", { name: "菜单" }));
    await userEvent.click(await screen.findByRole("menuitem", { name: "删除" }));
    expect(await screen.findByRole("alertdialog")).toBeVisible();
    await userEvent.keyboard("{Escape}");
    await waitFor(() => expect(screen.getByRole("button", { name: "菜单" })).toHaveFocus());
  });
  it("portals inherit the same fixed light theme instead of falling back to body", async () => {
    render(<NeuroProvider mode="light"><Dialog title="浅色弹窗" trigger={<Button>打开</Button>}>内容</Dialog></NeuroProvider>);
    await userEvent.click(screen.getByRole("button", { name: "打开" }));
    expect((await screen.findByRole("dialog")).closest("[data-neuro-mode]")).toHaveAttribute("data-neuro-mode", "light");
  });
  it("menus are keyboard reachable and execute only enabled items", async () => {
    const choose = vi.fn();
    render(wrap(<DropdownMenu label="操作" trigger={<Button>菜单</Button>} items={[{ id: "one", label: "执行", onSelect: choose }, { id: "two", label: "不可用", disabled: true, onSelect: choose }]} />));
    const user = userEvent.setup();
    screen.getByRole("button", { name: "菜单" }).focus();
    await user.keyboard("{Enter}");
    expect(await screen.findByRole("menuitem", { name: "执行" })).toHaveFocus();
    await user.keyboard("{Enter}");
    expect(choose).toHaveBeenCalledOnce();
  });
  it("tabs support directional navigation with one active tab", async () => {
    function Example() { const [value, setValue] = useState("a"); return <Tabs label="分区" value={value} onValueChange={setValue} items={[{ value: "a", label: "概览", content: "概览内容" }, { value: "b", label: "属性", content: "属性内容" }]} />; }
    render(wrap(<Example />));
    screen.getByRole("tab", { name: "概览" }).focus();
    await userEvent.keyboard("{ArrowRight}");
    await waitFor(() => expect(screen.getByRole("tab", { name: "属性" })).toHaveAttribute("aria-selected", "true"));
    expect(screen.getByRole("tabpanel")).toHaveTextContent("属性内容");
  });
  it("tooltip uses the matching provider portal", async () => {
    render(wrap(<ActionTooltip label="提示内容"><Button>提示</Button></ActionTooltip>));
    await userEvent.hover(screen.getByRole("button", { name: "提示" }));
    expect((await screen.findByRole("tooltip")).closest(".neuro-root")).not.toBeNull();
  });
});

describe("numeric fields", () => {
  it.each([[9.6, 1, 12, 1, 10], [1.13, 0.25, 4, 0.25, 1.25], [-3, 1, 12, 1, 1], [99, 1, 12, 1, 12], [0.0000003, 0.0000001, 0.000001, 0.0000001, 0.0000003]])("normalizes %s within the declared range", (value, min, max, step, expected) => {
    expect(normalizeNumber(value, min, max, step)).toBe(expected);
  });
  it("exposes invalid numeric contracts rather than silently inventing a range", () => {
    expect(() => normalizeNumber(1, 2, 1, 1)).toThrow(RangeError);
    expect(() => normalizeNumber(1, 0, 1, 0)).toThrow(RangeError);
  });
  it("decimal stepping does not drift and boundary buttons are disabled", async () => {
    function Example() { const [value, setValue] = useState<number | null>(0.1); return <NumberField label="宽度" value={value} onValueChange={setValue} min={0.1} max={0.9} step={0.1} />; }
    render(wrap(<Example />));
    expect(screen.getByRole("button", { name: "减少 宽度" })).toBeDisabled();
    for (let index = 0; index < 8; index++) await userEvent.click(screen.getByRole("button", { name: "增加 宽度" }));
    expect(screen.getByRole("spinbutton")).toHaveValue(0.9);
    expect(screen.getByRole("button", { name: "增加 宽度" })).toBeDisabled();
  });
  it("required empty edits restore the committed value; optional empty edits emit null", () => {
    const changed = vi.fn();
    const { rerender } = render(wrap(<NumberField label="列数" value={4} onValueChange={changed} min={1} max={12} step={1} />));
    fireEvent.change(screen.getByRole("spinbutton"), { target: { value: "" } });
    fireEvent.blur(screen.getByRole("spinbutton"));
    expect(screen.getByRole("spinbutton")).toHaveValue(4);
    expect(changed).not.toHaveBeenCalled();
    rerender(wrap(<NumberField label="列数" value={4} onValueChange={changed} min={1} max={12} step={1} allowEmpty />));
    fireEvent.change(screen.getByRole("spinbutton"), { target: { value: "" } });
    fireEvent.blur(screen.getByRole("spinbutton"));
    expect(changed).toHaveBeenCalledWith(null);
  });
});

describe("clipboard and feedback lifetime", () => {
  it("copies the explicit value, blocks duplicate requests, and waits for real success", async () => {
    let finish!: () => void;
    const write = vi.fn(() => new Promise<void>(resolve => { finish = resolve; }));
    render(wrap(<CopyValue value="full-value" displayValue="display-value" label="复制" writeText={write} />));
    fireEvent.click(screen.getByRole("button")); fireEvent.click(screen.getByRole("button"));
    expect(write).toHaveBeenCalledExactlyOnceWith("full-value");
    expect(screen.getByRole("status")).not.toHaveTextContent("已复制");
    await act(async () => finish());
    expect(screen.getByRole("status")).toHaveTextContent("已复制");
  });
  it("failed copying is visible and never announces success", async () => {
    render(wrap(<CopyValue value="value" label="复制" writeText={async () => { throw new Error("Permission denied"); }} />));
    fireEvent.click(screen.getByRole("button"));
    await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("复制失败"));
    expect(screen.getByRole("status")).not.toHaveTextContent("已复制");
  });
  it("unmount releases copy-feedback timers", async () => {
    vi.useFakeTimers();
    const { unmount } = render(wrap(<CopyValue value="value" label="复制" writeText={async () => {}} />));
    fireEvent.click(screen.getByRole("button"));
    await act(async () => {});
    unmount();
    expect(vi.getTimerCount()).toBe(0);
  });
  it("toasts belong to their provider and release timers on unmount", () => {
    vi.useFakeTimers();
    function Trigger() { const notify = useToast(); return <Button onClick={() => notify({ message: "通知", tone: "success" })}>通知</Button>; }
    const { unmount, container } = render(<><NeuroProvider><ToastProvider><Trigger /></ToastProvider></NeuroProvider><NeuroProvider><ToastProvider><span>第二棵树</span></ToastProvider></NeuroProvider></>);
    fireEvent.click(screen.getByRole("button", { name: "通知" }));
    const providers = container.querySelectorAll(".neuro-root");
    expect(within(providers[0] as HTMLElement).getByRole("status")).toHaveTextContent("通知");
    expect(within(providers[1] as HTMLElement).queryByRole("status")).toBeNull();
    unmount();
    expect(vi.getTimerCount()).toBe(0);
  });
});
