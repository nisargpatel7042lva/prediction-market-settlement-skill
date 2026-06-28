# prediction-market-settlement-skill

skill: ./skill/SKILL.md

## About this repo

This repo contains the `prediction-market-settlement-skill` for Claude Code — an addon to `solana-dev-skill` covering safe settlement of prediction markets and outcome-resolved programs on Solana.

The `skill/` directory is the distributable artifact. The rest of the repo (README, install scripts, agents, commands) is scaffolding.

## When reviewing skill content

- Code snippets in `skill/*.md` are **illustrative patterns**, not copy-paste-ready programs. They use `// Illustrative Anchor sketch` headers.
- Do not add vendor-specific API calls (named third-party data providers, specific oracle product IDs) to the core skill files. If needed for a project-local example, add a separate file outside `skill/`.
- The `MarketState` enum is the canonical state machine definition — it lives in `dispute-windows.md` and is referenced (not redefined) elsewhere.
