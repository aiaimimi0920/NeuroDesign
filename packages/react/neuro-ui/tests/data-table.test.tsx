import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";
import { DataTable } from "../src/data-table";

it("keeps keyed data and detail rows stable across expansion and reorder", () => {
  const rows = [{ id: "a" }, { id: "b" }];
  const props = {
    label: "Plans", rows, getRowKey: (row: typeof rows[number]) => row.id,
    columns: [{ key: "name", header: "Name", rowHeader: true, cell: (row: typeof rows[number]) => row.id }]
  };
  const view = render(<DataTable {...props} rowDetails={row => ({ id: `details-${row.id}`, open: false, content: <input aria-label={`Draft ${row.id}`} defaultValue="retained" /> })} />);
  const header = screen.getByRole("rowheader", { name: "a" });
  const detail = document.getElementById("details-a");
  const input = screen.getByLabelText("Draft a");
  expect(detail).not.toBeVisible();
  view.rerender(<DataTable {...props} rows={[...rows].reverse()} rowDetails={row => ({ id: `details-${row.id}`, open: row.id === "a", content: <input aria-label={`Draft ${row.id}`} defaultValue="retained" /> })} />);
  expect(screen.getByRole("rowheader", { name: "a" })).toBe(header);
  expect(document.getElementById("details-a")).toBe(detail);
  expect(detail).toBeVisible();
  expect(screen.getByLabelText("Draft a")).toBe(input);
  expect(detail?.querySelector("td")).toHaveAttribute("colspan", "1");
});

it("includes the optional selection column in the detail span", () => {
  render(<DataTable label="Selectable plans" rows={["a"]} getRowKey={row => row}
    columns={[{ key: "name", header: "Name", cell: row => row }]}
    onSelectedKeysChange={() => { }} rowDetails={row => ({ id: `selectable-${row}`, open: true, content: "Detail" })} />);
  expect(document.getElementById("selectable-a")?.querySelector("td")).toHaveAttribute("colspan", "2");
});
