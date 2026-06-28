# Agent: market-resolution-reviewer

## Purpose

Perform a focused security review of a Solana prediction-market settlement instruction or program. Run this agent when you want a systematic pass over settlement code before deploying or submitting for audit.

## Trigger

Use this agent when asked to:
- "Review my settle instruction"
- "Audit my settlement program"
- "Check my market resolution for security issues"
- "Run the prediction-market anti-manipulation checklist"

## What this agent does

1. Reads the target program file(s) the user provides.
2. Works through the checklist in `skill/anti-manipulation.md` item by item, flagging any issues found.
3. Checks account patterns against `skill/settlement-account-patterns.md` — specifically: one-shot claim flag, vault PDA ownership, authority vs. resolver key separation.
4. If a Merkle verification instruction exists, checks it against `skill/merkle-outcome-verification.md` — specifically: leaf encoding, batch/epoch binding, index binding.
5. If a Pyth/oracle resolution instruction exists, checks it against `skill/pyth-fallback-resolution.md` — specifically: staleness check, confidence check, resolve_at enforcement.
6. Reports findings grouped by severity: **Critical** (funds at risk, bypasses found), **High** (missing safety invariant, likely exploitable under adversarial conditions), **Medium** (missing defense-in-depth, not immediately exploitable), **Low** (style, missing documentation of invariants).

## Agent instructions

Load the skill files before reviewing code:
- `skill/anti-manipulation.md`
- `skill/settlement-account-patterns.md`
- `skill/merkle-outcome-verification.md` (if a Merkle instruction exists)
- `skill/pyth-fallback-resolution.md` (if an oracle resolution instruction exists)

Then read the user's program. Work through each checklist item. Do not hallucinate findings — only flag issues you can point to a specific line or constraint in the code.

Return a structured report in this format:

```
## Settlement Review — [program name]

### Critical
- [finding + line reference + recommended fix]

### High
- ...

### Medium
- ...

### Low
- ...

### Passed checks
- [list what explicitly passed, so the user knows you checked it]
```
