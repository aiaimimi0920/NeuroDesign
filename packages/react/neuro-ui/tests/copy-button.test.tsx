import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { CopyButton } from "../src/index";

afterEach(() => vi.useRealTimers());
it("waits for the real promise, blocks duplicate writes and resets only feedback", async () => {
  vi.useFakeTimers();
  let resolve!: () => void;
  const onCopy = vi.fn(() => new Promise<void>(done => { resolve = done; }));
  render(<CopyButton label="复制值" onCopy={onCopy}>复制</CopyButton>);
  const button = screen.getByRole("button", { name: "复制值" });
  const copyPath = button.querySelector("svg path")?.getAttribute("d");
  fireEvent.click(button);
  fireEvent.click(button);
  expect(onCopy).toHaveBeenCalledTimes(1);
  expect(button).toBeDisabled();
  expect(button).toHaveAttribute("aria-busy", "true");
  expect(screen.getByRole("status")).toBeEmptyDOMElement();
  await act(async () => resolve());
  expect(button).toBeEnabled();
  expect(button).toHaveAttribute("data-copy-state", "copied");
  expect(button.querySelector(".n-copy-rest")).toHaveTextContent(/^复制$/);
  expect(button.querySelector("svg path")?.getAttribute("d")).not.toBe(copyPath);
  expect(screen.getByRole("status")).toHaveTextContent("已复制");
  await act(async () => { vi.advanceTimersByTime(1800); });
  expect(button).toHaveAttribute("data-copy-state", "idle");
  expect(button.querySelector("svg path")).toHaveAttribute("d", copyPath);
});

it("announces a failed copy and permits a later retry without false success", async () => {
  const onCopy = vi.fn<() => Promise<void>>().mockRejectedValueOnce(new Error("denied")).mockResolvedValueOnce();
  render(<CopyButton label="复制值" errorLabel="复制被拒绝" onCopy={onCopy} />);
  const button = screen.getByRole("button", { name: "复制值" });
  await act(async () => fireEvent.click(button));
  expect(button).toHaveAttribute("data-copy-state", "error");
  expect(button).toHaveAttribute("title", "复制被拒绝");
  expect(screen.getByRole("status")).toHaveTextContent("复制被拒绝");
  await act(async () => fireEvent.click(button));
  expect(button).toHaveAttribute("data-copy-state", "copied");
});

it("does not schedule feedback after unmount or leave its reset timer behind", async () => {
  vi.useFakeTimers();
  let resolve!: () => void;
  const view = render(<CopyButton label="复制值" onCopy={() => new Promise<void>(done => { resolve = done; })} />);
  fireEvent.click(screen.getByRole("button"));
  view.unmount();
  await act(async () => resolve());
  expect(vi.getTimerCount()).toBe(0);
});
