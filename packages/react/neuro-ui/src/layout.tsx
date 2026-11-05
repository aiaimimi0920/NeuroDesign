import { useState, type HTMLAttributes, type ReactNode } from "react";
import { IconButton } from "./actions";
import { Icon } from "./icons";
import { Dialog } from "./overlays";
import { cx } from "./theme";

export function Panel({ className, ...props }: HTMLAttributes<HTMLElement>) {
  return <section {...props} className={cx("n-panel", className)} />;
}
export function Toolbar({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return <div {...props} className={cx("n-toolbar", className)} />;
}
export function Stack({ horizontal, className, ...props }: HTMLAttributes<HTMLDivElement> & { horizontal?: boolean }) {
  return <div {...props} className={cx(horizontal ? "n-row" : "n-stack", className)} />;
}
export interface AppShellProps {
  title: ReactNode;
  navigation?: ReactNode;
  navigationLabel?: string;
  actions?: ReactNode;
  toolbar?: ReactNode;
  inspector?: ReactNode;
  inspectorLabel?: string;
  status?: ReactNode;
  children: ReactNode;
}
/** Optional composition slots, not a required product definition. */
export function AppShell({ title, navigation, navigationLabel = "导航", actions, toolbar, inspector, inspectorLabel = "详情", status, children }: AppShellProps) {
  const [navigationOpen, setNavigationOpen] = useState(false);
  return <div className={cx("n-app-shell", !!navigation && "n-has-navigation")}>
    {navigation ? <aside className="n-app-navigation" aria-label={navigationLabel}>{navigation}</aside> : null}
    <main className="n-app-main">
      <header className="n-app-header"><div className="n-row">
        {navigation ? <div className="n-mobile-navigation"><Dialog title={navigationLabel} open={navigationOpen} onOpenChange={setNavigationOpen}
          trigger={<IconButton compact variant="quiet" label={navigationLabel}><Icon name="menu" /></IconButton>}>
          <div onClick={event => { if ((event.target as HTMLElement).closest("button, a")) setNavigationOpen(false); }}>{navigation}</div>
        </Dialog></div> : null}
        <h1>{title}</h1></div><div className="n-row">{actions}</div>
      </header>
      {toolbar ? <div className="n-app-toolbar">{toolbar}</div> : null}
      <div className={cx("n-app-workarea", !!inspector && "n-has-inspector")}>
        <div className="n-app-content">{children}</div>
        {inspector ? <aside className="n-app-inspector" aria-label={inspectorLabel}>{inspector}</aside> : null}
      </div>
      {status ? <footer className="n-app-status">{status}</footer> : null}
    </main>
  </div>;
}

export function NavigationItem({ active, className, ...props }: React.ComponentProps<"button"> & { active?: boolean }) {
  return <button {...props} type="button" className={cx("n-navigation-item", className)} aria-current={active ? "page" : undefined} />;
}
