import { useState } from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { Disclosure, Input, NeuroProvider } from "../src/index";

it("keeps an uncontrolled disclosure's state across parent rerenders and reports changes", async () => {
  const change = vi.fn();
  const tree = (label: string) => <NeuroProvider><Disclosure title="属性" defaultOpen onOpenChange={change}><Input aria-label={label} /></Disclosure></NeuroProvider>;
  const view = render(tree("名称")), user = userEvent.setup();
  const summary = screen.getByText("属性");
  expect(summary).toHaveAttribute("aria-expanded", "true");
  await user.click(summary);
  expect(summary).toHaveAttribute("aria-expanded", "false");
  expect(change).toHaveBeenCalledExactlyOnceWith(false);
  view.rerender(tree("标题"));
  expect(summary.closest("details")).not.toHaveAttribute("open");
  await user.click(summary);
  expect(summary).toHaveAttribute("aria-expanded", "true");
  expect(document.getElementById(summary.getAttribute("aria-controls")!)).toContainElement(screen.getByRole("textbox"));
});

it("lets the application control single-open state without remounting summaries", async () => {
  function Example() {
    const [open, setOpen] = useState<string | null>(null);
    return <>{["网络", "缓存"].map(value => <Disclosure key={value} title={<h2>{value}</h2>}
      open={open === value} onOpenChange={next => setOpen(next ? value : null)}>{open === value ? <Input aria-label={value} /> : null}</Disclosure>)}</>;
  }
  render(<NeuroProvider><Example /></NeuroProvider>);
  const user = userEvent.setup(), network = screen.getByText("网络").closest("summary")!, cache = screen.getByText("缓存").closest("summary")!;
  await user.click(network);
  expect(screen.getByRole("textbox", { name: "网络" })).toBeVisible();
  await user.click(cache);
  expect(network).toHaveAttribute("aria-expanded", "false");
  expect(cache).toHaveAttribute("aria-expanded", "true");
  expect(screen.queryByRole("textbox", { name: "网络" })).not.toBeInTheDocument();
  expect(screen.getByText("缓存").closest("summary")).toBe(cache);
  await user.click(cache);
  expect(cache).toHaveAttribute("aria-expanded", "false");
});

it("does not toggle a controlled disclosure until its owner accepts the change", async () => {
  const change = vi.fn();
  render(<NeuroProvider><Disclosure title="受控" open={false} onOpenChange={change}>内容</Disclosure></NeuroProvider>);
  await userEvent.setup().click(screen.getByText("受控"));
  expect(change).toHaveBeenCalledExactlyOnceWith(true);
  expect(screen.getByText("受控").closest("details")).not.toHaveAttribute("open");
});
