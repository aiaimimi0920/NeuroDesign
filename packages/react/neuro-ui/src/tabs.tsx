import { forwardRef, type ComponentPropsWithoutRef } from "react";
import * as RadixTabs from "@radix-ui/react-tabs";
import { cx } from "./theme";

/** Composable tabs let applications place toolbars between the list and panels. */
export const TabsRoot = forwardRef<HTMLDivElement, ComponentPropsWithoutRef<typeof RadixTabs.Root>>(function TabsRoot({ className, ...props }, ref) {
  return <RadixTabs.Root {...props} ref={ref} className={cx("n-tabs", className)} />;
});
export const TabList = forwardRef<HTMLDivElement, ComponentPropsWithoutRef<typeof RadixTabs.List> & { label: string }>(function TabList({ label, className, ...props }, ref) {
  return <RadixTabs.List {...props} ref={ref} aria-label={label} className={cx("n-tab-list", className)} />;
});
export const Tab = forwardRef<HTMLButtonElement, ComponentPropsWithoutRef<typeof RadixTabs.Trigger>>(function Tab({ className, ...props }, ref) {
  return <RadixTabs.Trigger {...props} ref={ref} className={cx("n-tab", className)} />;
});
export const TabPanel = forwardRef<HTMLDivElement, ComponentPropsWithoutRef<typeof RadixTabs.Content>>(function TabPanel({ className, ...props }, ref) {
  return <RadixTabs.Content {...props} ref={ref} className={cx("n-tab-content", className)} />;
});
