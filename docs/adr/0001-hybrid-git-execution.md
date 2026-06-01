# Hybrid Git execution: git2 for reads, shell-out for writes

Gitty uses `git2` (libgit2 Rust bindings) for all read operations (status, log, branch info, ref inspection) and shells out to the system `git` CLI for all write operations (pull, fetch, rebase, checkout, merge, stash, clean, reset).

Read operations are the backbone of dashboards, health checks, and change tracking — `git2` gives structured data without output parsing. Write operations need full compatibility with user git config, hooks, credentials, and edge-case git features that libgit2 doesn't cover (partial clone, sparse checkout, some rebase modes). This split is the same approach used by most serious Git GUIs (GitKraken, Tower).

The trade-off: users must have `git` installed for write operations. This is acceptable because Gitty's target audience (developers managing many repos) already has git installed. The alternative — pure `git2` — would sacrifice compatibility; pure shell-out would sacrifice performance and structured data access for reads.
