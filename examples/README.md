Repro for issue #21

Steps
1) Open this folder in Zed with the css-variables extension enabled.
2) Open main.scss and place the cursor after `var(--` to trigger completion.
3) Observe diagnostics for --accent-missing.
4) Edit vars.scss: rename --accent to --accent-missing and save.
5) Return to main.scss and edit a character; diagnostics/completions should update.

Expected behavior
- Completions show --accent/--accent-2/--muted.
- Diagnostics clear when the variable is defined.

If the issue reproduces, diagnostics do not update after edits and/or completion never appears in main.scss.
