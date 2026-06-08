import { createElement, ArrowLeft, Search, X } from "lucide";

export type BpIconName = "back" | "search" | "x";

const ICONS = {
  back: ArrowLeft,
  search: Search,
  x: X,
} as const;

const ICON_SIZE = {
  sm: "var(--font-sm)",
  md: "var(--font-md)",
} as const;

export function mountIcon(
  host: HTMLElement,
  name: BpIconName,
  size: keyof typeof ICON_SIZE = "md",
) {
  host.replaceChildren(
    createElement(ICONS[name], {
      width: ICON_SIZE[size],
      height: ICON_SIZE[size],
    }),
  );
}
