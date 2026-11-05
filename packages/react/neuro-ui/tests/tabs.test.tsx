import { useState } from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";
import { Input, NeuroProvider, Tab, TabList, TabPanel, TabsRoot } from "../src/index";

it("supports separated toolbars, native tab keyboard navigation and retained inactive panels", async () => {
  function Example() {
    const [value, setValue] = useState("services");
    return <TabsRoot asChild value={value} onValueChange={setValue}><section aria-label="工作区">
      <TabList label="MCP 工作区"><Tab value="services">服务</Tab><Tab value="store">商店</Tab><Tab value="blocked" disabled>不可用</Tab></TabList>
      <Input aria-label="搜索" />
      <TabPanel value="services" forceMount hidden={value !== "services"}><Input aria-label="名称" /></TabPanel>
      <TabPanel value="store" forceMount hidden={value !== "store"}>商店内容</TabPanel>
    </section></TabsRoot>;
  }
  const user = userEvent.setup();
  render(<NeuroProvider><Example /></NeuroProvider>);
  const field = screen.getByRole("textbox", { name: "名称" });
  await user.type(field, "未保存草稿");
  const services = screen.getByRole("tab", { name: "服务" });
  services.focus();
  await user.keyboard("{End}");
  const store = screen.getByRole("tab", { name: "商店" });
  expect(store).toHaveFocus();
  expect(store).toHaveAttribute("aria-selected", "true");
  expect(screen.getByRole("tabpanel")).toHaveAccessibleName("商店");
  expect(screen.getByRole("tabpanel")).toHaveAttribute("id", store.getAttribute("aria-controls"));
  expect(screen.queryByRole("textbox", { name: "名称" })).not.toBeInTheDocument();
  await user.keyboard("{Home}");
  expect(services).toHaveFocus();
  expect(screen.getByRole("textbox", { name: "名称" })).toBe(field);
  expect(field).toHaveValue("未保存草稿");
  await user.keyboard("{ArrowLeft}");
  expect(store).toHaveFocus();
});
