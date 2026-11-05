import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { SegmentedControl } from "../src/selection";

it("associates a group error without replacing its radios or focused selection", async () => {
  const onValueChange = vi.fn();
  const props = { label: "Runtime role", name: "runtime", value: "one", onValueChange, options: [{ value: "one", label: "One" }, { value: "two", label: "Two" }] };
  const view = render(<SegmentedControl {...props} error="Choose an available role" />);
  const group = screen.getByRole("radiogroup", { name: "Runtime role" });
  const radio = screen.getByRole("radio", { name: "One" });
  expect(group).toHaveAttribute("aria-invalid", "true");
  expect(group).toHaveAccessibleDescription("Choose an available role");
  await userEvent.click(radio);
  view.rerender(<SegmentedControl {...props} />);
  expect(screen.getByRole("radio", { name: "One" })).toBe(radio);
  expect(radio).toHaveFocus();
  expect(group).not.toHaveAttribute("aria-invalid");
  expect(group).not.toHaveAttribute("aria-describedby");
  expect(screen.queryByRole("alert")).toBeNull();
});

it("keeps diagnostics unique for separate groups", () => {
  const props = { value: "a", onValueChange() { }, options: [{ value: "a", label: "A" }] };
  render(<><SegmentedControl {...props} label="First" name="first" error="First error" /><SegmentedControl {...props} label="Second" name="second" error="Second error" /></>);
  expect(screen.getByRole("radiogroup", { name: "First" })).toHaveAccessibleDescription("First error");
  expect(screen.getByRole("radiogroup", { name: "Second" })).toHaveAccessibleDescription("Second error");
});
