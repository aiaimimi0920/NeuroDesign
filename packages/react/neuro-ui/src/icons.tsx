import type { SVGProps } from "react";

const paths = {
  plus: "M8 2v12M2 8h12",
  minus: "M2 8h12",
  close: "m3 3 10 10M13 3 3 13",
  check: "m2 8 4 4 8-8",
  chevron: "m4 6 4 4 4-4",
  copy: "M6 6h8v8H6zM10 4V2H2v8h2",
  search: "m11 11 4 4M12 7a5 5 0 1 1-10 0 5 5 0 0 1 10 0",
  folder: "M1 4h5l2 2h7v8H1zM1 4V2h5l2 2",
  grid: "M2 2h4v4H2zM10 2h4v4h-4zM2 10h4v4H2zM10 10h4v4h-4z",
  list: "M5 3h9M5 8h9M5 13h9M2 3h.1M2 8h.1M2 13h.1",
  more: "M3 8h.1M8 8h.1M13 8h.1",
  trash: "M2 4h12M6 4V2h4v2M4 4v10h8V4M7 7v4M9 7v4",
  star: "m8 1 2.1 4.3L15 6l-3.5 3.4.8 4.9L8 12l-4.3 2.3.8-4.9L1 6l4.9-.7z",
  info: "M8 7v5M8 4h.01M15 8A7 7 0 1 1 1 8a7 7 0 0 1 14 0",
  warning: "m8 1 7 13H1zM8 6v3M8 12h.01",
  menu: "M2 3h12M2 8h12M2 13h12",
} as const;

export type IconName = keyof typeof paths;
export function Icon({ name, className = "", ...props }: SVGProps<SVGSVGElement> & { name: IconName }) {
  return <svg {...props} className={`n-icon ${className}`} viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d={paths[name]} /></svg>;
}
