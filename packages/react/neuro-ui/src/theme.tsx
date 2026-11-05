import { createContext, useContext, useMemo, useState, type HTMLAttributes, type ReactNode } from "react";
import * as Tooltip from "@radix-ui/react-tooltip";

export type NeuroMode = "dark" | "light";
const ThemeContext = createContext<{ mode: NeuroMode; portal: HTMLDivElement | null } | null>(null);

export interface NeuroProviderProps extends HTMLAttributes<HTMLDivElement> {
  mode?: NeuroMode;
  inheritTypography?: boolean;
  children: ReactNode;
}

/** Fixed presets, scoped styles and a matching portal host; no application state. */
export function NeuroProvider({ mode = "dark", inheritTypography = false, children, className = "", ...props }: NeuroProviderProps) {
  const [portal, setPortal] = useState<HTMLDivElement | null>(null);
  const context = useMemo(() => ({ mode, portal }), [mode, portal]);
  return (
    <div {...props} className={cx("neuro-root", inheritTypography && "n-inherit-typography", className)} data-neuro-mode={mode}>
      <ThemeContext.Provider value={context}>
        <Tooltip.Provider delayDuration={350} skipDelayDuration={120}>
          {children}
          <div className="n-portal-host" ref={setPortal} />
        </Tooltip.Provider>
      </ThemeContext.Provider>
    </div>
  );
}

export function useNeuroTheme() {
  const context = useContext(ThemeContext);
  if (!context) throw new Error("Neuro components require a NeuroProvider");
  return context;
}

export function cx(...values: (string | false | undefined)[]) {
  return values.filter(Boolean).join(" ");
}
