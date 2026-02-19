## Issue Tracking

This project uses **bd (beads)** for issue tracking.
Run `bd prime` for workflow context, or install hooks (`bd hooks install`) for auto-injection.

**Quick reference:**
- `bd ready` - Find unblocked work
- `bd create "Title" --type task --priority 2` - Create issue
- `bd close <id>` - Complete work
- `bd sync` - Sync with git (run at session end)

For full workflow details: `bd prime`

## Portfolio Tracking in Linear (Required)

For cross-project status visibility, mirror work in Linear project `zed-css-variables` in addition to bd.

- Every work session must be mapped to a Linear issue.
- When work starts, move the Linear issue to `Started`.
- When work ends, update the issue with outcome and next step, then set `Backlog` or `Done`.
- Pull requests must reference a Linear issue ID.
- Do not cut a release or tag without confirming related Linear issues are up to date.

## Build Requirements

- Always use stable Rust - do not add dependencies that require nightly features
