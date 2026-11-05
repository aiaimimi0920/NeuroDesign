import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { Button, EmptyState, NeuroProvider } from "../src/index";

it("empty states retain root identity and standard semantics when density changes", () => {
  const { rerender } = render(<NeuroProvider><EmptyState title="没有内容" role="status" data-testid="empty" /></NeuroProvider>);
  const root = screen.getByRole("status");
  expect(root).toHaveClass("n-empty");
  expect(root).not.toHaveClass("n-empty--compact");
  rerender(<NeuroProvider><EmptyState compact title="当前范围为空" role="status" data-testid="empty" /></NeuroProvider>);
  expect(screen.getByRole("status")).toBe(root);
  expect(root).toHaveClass("n-empty--compact");
  expect(root).toHaveTextContent("当前范围为空");
});

it("empty-state actions remain ordinary explicit actions without implicit work", async () => {
  const click = vi.fn();
  const { rerender } = render(<NeuroProvider><EmptyState compact title="没有内容"><Button onClick={click}>创建</Button></EmptyState></NeuroProvider>);
  expect(click).not.toHaveBeenCalled();
  const button = screen.getByRole("button", { name: "创建" });
  await userEvent.click(button);
  expect(click).toHaveBeenCalledExactlyOnceWith(expect.anything());
  rerender(<NeuroProvider><EmptyState compact title="没有内容"><Button onClick={click} disabled>创建</Button></EmptyState></NeuroProvider>);
  expect(screen.getByRole("button", { name: "创建" })).toBe(button);
  expect(button).toBeDisabled();
});
