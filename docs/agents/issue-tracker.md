# Issue tracker: GitHub

Issues and PRDs for this repo live as GitHub issues. Use the `gh` CLI for all operations.

## Writing bodies — always `--body-file`

Never pass bodies inline (`--body "..."`) and never use a shell heredoc — PowerShell has no heredoc and mangles multi-line and quoted text. Write the body to a UTF-8 file first, then point `gh` at it:

- **Create**: `gh issue create --title "..." --label "..." --body-file body.md`
- **Comment**: `gh issue comment <number> --body-file comment.md`
- **Edit body**: `gh issue edit <number> --body-file body.md`

## Never round-trip a body through PowerShell

`gh ... | Out-File` (also `>` / `Add-Content`) **double-encodes UTF-8** on Windows PowerShell: em-dashes (`—`) and other non-ASCII become mojibake, and re-uploading that corrupts the issue body. Rules:

- **Reading** to stdout is safe: `gh issue view <n> --json body --jq .body`.
- To **modify** a body, rebuild the full body in a freshly written UTF-8 file (do NOT pipe it from `gh`) and upload with `--body-file`.
- After an edit, **verify** with a UTF-8 console: `[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; gh issue view <n> --json body --jq .body`.

## Other conventions

- **Read an issue**: `gh issue view <number> --comments`.
- **List issues**: `gh issue list --state open --json number,title,labels` (add `--label` / `--state` filters). When using `--jq`, quote the expression carefully in PowerShell (spaces and `#` need quoting) or prefer `--json` and parse the result.
- **Apply / remove labels**: `gh issue edit <number> --add-label "..."` / `--remove-label "..."`.
- **Close**: `gh issue close <number>`. For a multi-line closing note, post it first via `gh issue comment <number> --body-file note.md`, then close. A short single-line `--comment "..."` inline is fine.

Infer the repo from `git remote -v` — `gh` does this automatically when run inside a clone.

## When a skill says "publish to the issue tracker"

Create a GitHub issue (see "Writing bodies" above).

## When a skill says "fetch the relevant ticket"

Run `gh issue view <number> --comments`.
