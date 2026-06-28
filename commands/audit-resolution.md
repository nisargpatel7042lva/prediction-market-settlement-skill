# /audit-resolution

Run the market-resolution-reviewer agent on the current project's settlement program.

## Usage

```
/audit-resolution [path/to/program.rs]
```

If no path is given, look for Anchor program files in `programs/*/src/lib.rs` or `programs/*/src/instructions/settle*.rs`.

## What it does

Invokes the `market-resolution-reviewer` agent (see `agents/market-resolution-reviewer.md`) on the specified file(s). Returns a structured security report grouped by severity: Critical, High, Medium, Low, and Passed checks.

## Example

```
/audit-resolution programs/epoch/src/instructions/settle_market.rs
```
