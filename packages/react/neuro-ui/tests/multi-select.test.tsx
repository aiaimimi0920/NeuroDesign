import { useState } from "react";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { Button, Dialog, MultiSelect, NeuroProvider, Notice } from "../src/index";

const options = [
  { value: "a", label: "基础", description: "group-a", meta: "2 个账号", searchText: "基础 group-a" },
  { value: "b", label: "高级", meta: "3 个账号" },
  { value: "c", label: "不可用", disabled: true },
];
function Example({ disabled = false }: { disabled?: boolean }) {
  const [values, setValues] = useState<string[]>([]);
  return <MultiSelect label="权益" options={options} values={values} onValuesChange={setValues} disabled={disabled}
    search={{ label: "搜索权益", emptyMessage: "没有匹配的权益" }} />;
}

it("searches labels and explicit identifiers, navigates enabled options and keeps selection nodes", async () => {
  const user = userEvent.setup();
  render(<NeuroProvider><Example /></NeuroProvider>);
  await user.click(screen.getByRole("button", { name: /权益，/ }));
  const search = screen.getByRole("searchbox");
  expect(search).toHaveFocus();
  await user.type(search, "group-a");
  const option = screen.getByRole("checkbox");
  await user.keyboard("{ArrowDown} ");
  expect(option).toHaveFocus();
  expect(option).toBeChecked();
  expect(screen.getByRole("checkbox")).toBe(option);
  await user.clear(search);
  await user.keyboard("{End}");
  expect(search).toHaveFocus();
  await user.keyboard("{ArrowUp}");
  expect(screen.getByRole("checkbox", { name: /高级/ })).toHaveFocus();
  await user.keyboard("{Home}");
  expect(option).toHaveFocus();
  await user.type(search, "unknown");
  expect(screen.getByRole("status")).toHaveTextContent("没有匹配的权益");
});

it("consumes the first Escape inside a modal, restores both return targets and inherits its theme", async () => {
  function Modal() {
    const [open, setOpen] = useState(false);
    return <Dialog title="编辑" trigger={<Button>打开</Button>} open={open} onOpenChange={setOpen}><Example /></Dialog>;
  }
  const onEscape = vi.fn();
  const user = userEvent.setup();
  render(<NeuroProvider mode="light"><div onKeyDown={onEscape}><Modal /></div></NeuroProvider>);
  const open = screen.getByRole("button", { name: "打开" });
  await user.click(open);
  const trigger = screen.getByRole("button", { name: /权益，/ });
  await user.click(trigger);
  const popup = screen.getByRole("dialog", { name: "权益" });
  expect(popup.closest("[data-neuro-mode]")).toHaveAttribute("data-neuro-mode", "light");
  onEscape.mockClear();
  await user.keyboard("{Escape}");
  await waitFor(() => expect(popup).not.toBeInTheDocument());
  expect(screen.getByRole("dialog", { name: "编辑" })).toBeInTheDocument();
  expect(trigger).toHaveFocus();
  expect(onEscape).not.toHaveBeenCalled();
  await user.keyboard("{Escape}");
  await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  expect(open).toHaveFocus();
});

it("closes immediately on disabled state and clears transient search before reopening", async () => {
  const user = userEvent.setup();
  const view = render(<NeuroProvider><Example /></NeuroProvider>);
  await user.click(screen.getByRole("button", { name: /权益，/ }));
  await user.type(screen.getByRole("searchbox"), "group-a");
  view.rerender(<NeuroProvider><Example disabled /></NeuroProvider>);
  expect(screen.queryByRole("searchbox")).not.toBeInTheDocument();
  view.rerender(<NeuroProvider><Example /></NeuroProvider>);
  await user.click(screen.getByRole("button", { name: /权益，/ }));
  expect(screen.getByRole("searchbox")).toHaveValue("");
  expect(screen.getAllByRole("checkbox")).toHaveLength(3);
});

it("renders a caller-owned empty or unsupported state without claiming success", async () => {
  render(<NeuroProvider><MultiSelect label="权益" options={[]} values={[]} onValuesChange={() => {}}
    empty={<Notice tone="danger">服务不支持此功能</Notice>} /></NeuroProvider>);
  await userEvent.click(screen.getByRole("button", { name: /权益，/ }));
  expect(screen.getByRole("alert")).toHaveTextContent("服务不支持此功能");
});

it("lets native Tab leave the last choice instead of looping within the popup", async () => {
  const user = userEvent.setup();
  render(<NeuroProvider><Example /><Button>下一项</Button></NeuroProvider>);
  await user.click(screen.getByRole("button", { name: /权益，/ }));
  await user.keyboard("{ArrowDown}{End}{Tab}");
  await waitFor(() => expect(screen.queryByRole("searchbox")).not.toBeInTheDocument());
  expect(screen.getByRole("button", { name: "下一项" })).toHaveFocus();
});
