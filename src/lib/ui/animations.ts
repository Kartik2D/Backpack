import { cubicOut } from "svelte/easing";
import type { TransitionConfig } from "svelte/transition";

/** Shared timing for UI transitions (ms). */
export const DURATION = {
  fast: 120,
  base: 180,
} as const;

type FadeParams = {
  delay?: number;
  duration?: number;
};

/** Fade opacity in/out. Use for dark backgrounds / overlays. */
export function fade(node: Element, { delay = 0, duration = DURATION.base }: FadeParams = {}): TransitionConfig {
  const target = +getComputedStyle(node).opacity;
  return {
    delay,
    duration,
    easing: cubicOut,
    css: (t) => `opacity: ${t * target}`,
  };
}

type ZoomParams = {
  delay?: number;
  duration?: number;
  /** Scale to start from when entering (and grow to when leaving). */
  start?: number;
  /** Opacity to start from when entering. */
  opacity?: number;
};

/** Zoom (scale) + fade in/out. Use for items: modals, menus, toasts, cards. */
export function zoom(
  node: Element,
  { delay = 0, duration = DURATION.base, start = 1.08, opacity = 0 }: ZoomParams = {},
): TransitionConfig {
  const style = getComputedStyle(node);
  const targetOpacity = +style.opacity;
  const baseTransform = style.transform === "none" ? "" : style.transform;
  const scaleDelta = 1 - start;
  const opacityDelta = targetOpacity * (1 - opacity);
  return {
    delay,
    duration,
    easing: cubicOut,
    css: (t) =>
      `transform: ${baseTransform} scale(${start + t * scaleDelta}); opacity: ${opacity + t * opacityDelta}`,
  };
}
