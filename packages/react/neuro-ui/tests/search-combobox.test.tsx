import { useRef, useState } from "react";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { Button, Dialog, NeuroProvider, SearchCombobox, SegmentedControl } from "../src/index";

const options = [
  { value: "a", label: "外观", description: "常规" },
  { value: "b", label: "不可用", disabled: true },
  { value: "c", label: "日志", description: "诊断" },
];
function Example({ disabled = false, onSelect = () => {}, onExpandedChange }: {
  disabled?: boolean; onSelect?: (value: string) => void; onExpandedChange?: (expanded: boolean) => void;
}) {
  const [query, setQuery] = useState("");
  return <SearchCombobox label="查找设置" value={query} onValueChange={setQuery} disabled={disabled}
    options={query === "missing" ? [] : options} onSelect={onSelect} onExpandedChange={onExpandedChange} />;
}
it("keeps input DOM and focus, skips disabled suggestions and selects by keyboard", async () => {
  const selected = vi.fn(), expanded = vi.fn();
  render(<NeuroProvider><Example onSelect={selected} onExpandedChange={expanded} /></NeuroProvider>);
  const user = userEvent.setup(), input = screen.getByRole("combobox");
  await user.type(input, "日志");
  expect(input).toHaveFocus();
  expect(expanded).toHaveBeenLastCalledWith(true);
  expect(document.getElementById(input.getAttribute("aria-activedescendant")!)).toHaveTextContent("外观");
  await user.keyboard("{ArrowDown}{Enter}");
  expect(selected).toHaveBeenCalledExactlyOnceWith("c");
  expect(screen.getByRole("combobox")).toBe(input);
  expect(input).toHaveFocus();
  await waitFor(() => expect(screen.queryByRole("listbox")).not.toBeInTheDocument());
});
it("clears query on Escape, exposes empty feedback and returns to input after clear", async () => {
  render(<NeuroProvider><Example /></NeuroProvider>);
  const user = userEvent.setup(), input = screen.getByRole("combobox");
  await user.type(input, "missing");
  expect(screen.getByRole("status")).toHaveTextContent("没有匹配结果");
  await user.keyboard("{Escape}");
  expect(input).toHaveValue("");
  await user.type(input, "外观");
  await user.click(screen.getByRole("button", { name: "清空搜索" }));
  expect(input).toHaveFocus(); expect(input).toHaveValue("");
  expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
});
it("lets Tab leave suggestions and closes on disabled without reopening stale state", async () => {
  const expanded = vi.fn();
  const tree = (disabled: boolean) => <NeuroProvider><Example disabled={disabled} onExpandedChange={expanded} /><Button>下一项</Button></NeuroProvider>;
  const view = render(tree(false));
  const user = userEvent.setup(), input = screen.getByRole("combobox");
  await user.type(input, "外观");
  await user.tab();
  expect(screen.getByRole("button", { name: "清空搜索" })).toHaveFocus();
  expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
  await user.click(input); view.rerender(tree(true));
  expect(input).toBeDisabled(); expect(expanded).toHaveBeenLastCalledWith(false);
  view.rerender(tree(false));
  expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
});
it("does not steal focus from the caller's destination after selection", async () => {
  function Destination() {
    const target = useRef<HTMLButtonElement>(null);
    return <><Example onSelect={() => target.current?.focus()} /><Button ref={target}>定位目标</Button></>;
  }
  render(<NeuroProvider mode="light"><Destination /></NeuroProvider>);
  const user = userEvent.setup();
  await user.type(screen.getByRole("combobox"), "外观");
  expect(screen.getByRole("listbox").closest("[data-neuro-mode]")).toHaveAttribute("data-neuro-mode", "light");
  await user.click(screen.getByRole("option", { name: "外观" }));
  await waitFor(() => expect(screen.getByRole("button", { name: "定位目标" })).toHaveFocus());
});
it("consumes nested Escape before closing the parent dialog and preserves return focus", async () => {
  function Modal() {
    const [open, setOpen] = useState(false);
    return <Dialog title="设置" open={open} onOpenChange={setOpen} trigger={<Button>打开设置</Button>}><Example /></Dialog>;
  }
  render(<NeuroProvider><Modal /></NeuroProvider>);
  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "打开设置" }));
  await user.type(screen.getByRole("combobox"), "外观");
  await user.keyboard("{Escape}");
  expect(screen.getByRole("dialog", { name: "设置" })).toBeVisible();
  expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
  await user.keyboard("{Escape}");
  await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  await waitFor(() => expect(screen.getByRole("button", { name: "打开设置" })).toHaveFocus());
});
it("keeps a hidden group name accessible while retaining native radio behavior", async () => {
  function Group() {
    const [value, setValue] = useState("a");
    return <SegmentedControl hideLabel name="mode" label="模式" options={options.filter(option => !option.disabled)} value={value} onValueChange={setValue} />;
  }
  render(<NeuroProvider><Group /></NeuroProvider>);
  const group = screen.getByRole("radiogroup", { name: "模式" });
  expect(group.querySelector("legend")).toHaveClass("n-sr-only");
  await userEvent.click(screen.getByRole("radio", { name: "日志" }));
  expect(screen.getByRole("radio", { name: "日志" })).toBeChecked();
});
