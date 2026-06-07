/** Map a spacing scale index (1–6) to the corresponding CSS token. */
export function space(n: number) {
  return `var(--space-${n})`;
}
