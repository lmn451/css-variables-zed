export const theme = {
  accent: "var(--accent)",
  muted: "var(--muted)",
};

export function inject() {
  const css = `
    .injected {
      color: var(--accent);
    }
  `;
  return css;
}
