# Privacy

Gitty is built with a privacy-first architecture. Here is our commitment:

## Zero Data Collection

Gitty is fully offline. Your data never leaves your machine. There are no accounts, no telemetry, no analytics, no cloud sync.

## Network Activity

Network calls are only made to:

- **Your own Git remotes** — when fetching, pulling, or pushing repositories
- **Liveness endpoints you explicitly configure** — for optional health-check monitoring

Gitty never contacts any server operated by the Gitty project. There is no update phone-home, no crash reporting, and no usage analytics.

## Local-Only Storage

All configuration, repository metadata, and workspace state are stored locally on your machine. Nothing is synced to any cloud service.

## Open Source

Gitty is fully open source under the MIT license. You can audit every line of code at [github.com/caiomestres/gitty](https://github.com/caiomestres/gitty).
