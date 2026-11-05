import { createRef } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Button, ButtonLink, NeuroProvider } from "../src";

describe("ButtonLink", () => {
  it("shares opt-in wrapping without forwarding the appearance prop to native elements", () => {
    render(<NeuroProvider><Button wrap>Continue importing this account</Button><ButtonLink href="#section" wrap>Open account authorization</ButtonLink></NeuroProvider>);
    for (const control of [screen.getByRole("button"), screen.getByRole("link")]) {
      expect(control).toHaveClass("n-button-wrap");
      expect(control).not.toHaveAttribute("wrap");
    }
  });
  it("preserves native navigation attributes, events, children and the anchor ref", () => {
    const ref = createRef<HTMLAnchorElement>();
    const click = vi.fn(event => event.preventDefault());
    render(<NeuroProvider><ButtonLink ref={ref} href="https://example.test/authorize" target="_blank"
      rel="noopener noreferrer" onClick={click} className="host-link" compact>Open authorization</ButtonLink></NeuroProvider>);
    const link = screen.getByRole("link", { name: "Open authorization" });
    expect(ref.current).toBe(link);
    expect(link).toHaveAttribute("href", "https://example.test/authorize");
    expect(link).toHaveAttribute("target", "_blank");
    expect(link).toHaveAttribute("rel", "noopener noreferrer");
    expect(link).toHaveClass("n-button", "n-button-link", "n-button--secondary", "n-compact", "host-link");
    expect(link).not.toHaveAttribute("role");
    expect(link).not.toHaveAttribute("tabindex");
    fireEvent.click(link);
    expect(click).toHaveBeenCalledOnce();
  });

  it.each(["primary", "quiet", "danger", "positive"] as const)("shares the %s appearance without button-only behavior", variant => {
    render(<NeuroProvider><ButtonLink href="#section" variant={variant}>Go</ButtonLink></NeuroProvider>);
    const link = screen.getByRole("link");
    expect(link).toHaveClass(`n-button--${variant}`);
    expect(link).not.toHaveAttribute("type");
    expect(link).not.toHaveAttribute("aria-disabled");
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});
