# Liveness

Liveness monitors HTTP endpoints associated with your repositories. While **Health** checks Git state, **Liveness** checks whether running services are accessible.

## Overview

Liveness answers: "Is the service for this repository running and reachable?"

This is particularly useful for:
- Web applications with local dev servers
- Microservices in a workspace
- Docker-compose projects
- Backend services paired with frontend repos

## Liveness vs Health

| Aspect | Health | Liveness |
|--------|--------|----------|
| **Checks** | Git repository state | HTTP endpoint availability |
| **Examples** | Fresh, dirty, detached | `localhost:3000/health` |
| **Affects score** | Yes (Workspace Health) | No (separate concern) |
| **Independence** | Repo can be healthy | Service can be up |
| | but service down | but repo dirty |

A repository can be:
- **Healthy + Up** — Good state, service running
- **Healthy + Down** — Good state, service stopped
- **Critical + Up** — Needs Git attention, but service running
- **Critical + Down** — Both need attention

## Environment

Liveness is configured per **Environment** — a named HTTP endpoint associated with a repository.

### Environment Properties

```json
{
  "name": "dev",
  "endpoint": "http://localhost:3000/health",
  "interval_seconds": 60
}
```

| Field | Description | Default |
|-------|-------------|---------|
| `name` | Display name for the environment | Required |
| `endpoint` | Full URL to probe | Required |
| `interval_seconds` | Seconds between probes | Global default |

### Endpoint Discovery

Gitty can auto-discover endpoints by scanning repository files:

- `docker-compose.yml` — Extracts port mappings
- `Dockerfile` — Looks for `EXPOSE` directives
- `.env` — Finds `PORT` or `*_PORT` variables
- `package.json` — Reads `scripts.start` for ports
- Kubernetes manifests — Service port definitions

Auto-discovered endpoints are suggestions — you confirm or modify before activation.

## Probe Behavior

### HTTP Request

Gitty sends a simple HTTP GET to the configured endpoint:

```
GET /health HTTP/1.1
Host: localhost:3000
User-Agent: Gitty/Liveness-Probe
Accept: */*
Connection: close
```

### Success Criteria

| Aspect | Requirement |
|--------|-------------|
| **Connection** | TCP connection succeeds |
| **HTTP status** | Any 2xx or 3xx response |
| **Timeout** | 5 seconds |

Any connection failure, 4xx/5xx response, or timeout marks the endpoint as **Down**.

### Status Values

| Status | Meaning | Display |
|--------|---------|---------|
| `up` | Last probe succeeded | 🟢 Green dot |
| `down` | Last probe failed | 🔴 Red dot |
| `unknown` | No probe yet or probe disabled | ⚪ Gray dot |
| `skipped` | Repository is Missing | ⚪ Gray dot |

## Configuration

### Global Settings

Default behavior for all liveness probes:

```json
{
  "liveness": {
    "enabled": true,
    "default_interval_seconds": 60,
    "notification_on_failure": false
  }
}
```

| Setting | Description | Default |
|---------|-------------|---------|
| `enabled` | Master switch for liveness | `true` |
| `default_interval_seconds` | Probe interval | `60` |
| `notification_on_failure` | Alert on probe failure | `false` |

### Per-Repository

Each repository can have multiple environments:

```json
{
  "uuid": "...",
  "name": "myapp",
  "environments": [
    {
      "name": "dev",
      "endpoint": "http://localhost:3000/health",
      "interval_seconds": 30
    },
    {
      "name": "storybook",
      "endpoint": "http://localhost:6006",
      "interval_seconds": 120
    }
  ]
}
```

Per-environment `interval_seconds` overrides the global default.

### GUI Configuration

1. Navigate to a repository's detail page
2. Click **Add Environment** in the Liveness section
3. Enter name and endpoint URL
4. Optionally adjust probe interval
5. Save

## Liveness Dashboard

### Repository Cards

On the main Dashboard, repository cards show:

- **Liveness dot** — Small colored indicator (green/red/gray)
- **Tooltip** — Status text on hover
- **Detail link** — Click to see full probe history

### Liveness Indicators

Small status indicators appear in:

- Dashboard repository cards
- Repository detail page header
- Group tree (optional, if space permits)
- Changes view (when relevant)

### Probe History

Click a liveness dot to see:

- Current status and last probe time
- Response time history
- Recent failures with error details
- Uptime percentage (24h, 7d)

## Scheduler Integration

Liveness probes run as part of the scheduler's tick cycle:

```
Scheduler Loop:
1. Check if macro should run → Execute if needed
2. Check if liveness probes should run → Probe if needed
3. Update health if conditions met
4. Generate notifications if triggered
5. Sleep until next tick
```

Probes run independently from macros — a repository can be probed even if no macro targets it.

### Probe Scheduling

Each environment tracks its own `last_probe_timestamp`:

```
if (now - last_probe > interval_seconds):
    execute_probe()
    update_timestamp()
```

This ensures probes are distributed and don't create thundering herds.

## Notifications

Optional alerts when endpoints go down:

```json
{
  "liveness": {
    "notification_on_failure": true
  }
}
```

When enabled:
- First failure triggers a notification
- Repeated failures don't spam (deduplication window: 15 minutes)
- Recovery notifications optional (configurable)

**Delivery:**
- In-app notification panel
- OS-native toast (if supported and enabled)

## Use Cases

### Web Development

Monitoring local dev servers:

```json
{
  "environments": [
    { "name": "frontend", "endpoint": "http://localhost:5173" },
    { "name": "backend", "endpoint": "http://localhost:3000/health" }
  ]
}
```

### Docker-Compose Projects

Auto-discovered from `docker-compose.yml`:

```yaml
# docker-compose.yml
services:
  web:
    ports:
      - "3000:3000"  # Gitty suggests: http://localhost:3000
  db:
    ports:
      - "5432:5432"  # Gitty suggests: localhost:5432 (non-HTTP, user confirms)
```

### Microservices

Multiple related services in one workspace:

```
workspace/
├── api-gateway/      → http://localhost:8080
├── user-service/     → http://localhost:8081/health
├── payment-service/  → http://localhost:8082/health
└── web-app/          → http://localhost:3000
```

### Documentation Sites

Monitoring local preview servers:

```json
{
  "environments": [
    { "name": "mkdocs", "endpoint": "http://localhost:8000" },
    { "name": "storybook", "endpoint": "http://localhost:6006" }
  ]
}
```

## Best Practices

### Endpoint Selection

1. **Use health endpoints** — `/health`, `/status`, `/ready`
2. **Avoid heavy routes** — Don't hit database-intensive endpoints
3. **Lightweight responses** — JSON with status is fine; full HTML pages are wasteful
4. **Match your app** — Use the same endpoint your orchestrator uses

### Interval Tuning

| Scenario | Recommended Interval |
|----------|---------------------|
| Dev server hot reload | 10-30 seconds |
| Stable local service | 60 seconds |
| Remote staging | 60-300 seconds |
| Production (if monitored) | 60 seconds |

### Multiple Environments

Name environments clearly:

```json
{
  "environments": [
    { "name": "dev", "endpoint": "http://localhost:3000" },
    { "name": "docs", "endpoint": "http://localhost:8080" },
    { "name": "storybook", "endpoint": "http://localhost:6006" }
  ]
}
```

### Avoiding Overload

1. **Don't probe production from dev machines** — Use separate monitoring
2. **Respect intervals** — Don't set 1-second probes for dozens of repos
3. **Skip missing repos** — Liveness automatically skipped for Missing repositories

## Troubleshooting

### Endpoint always shows "unknown"

1. Check that liveness is enabled globally
2. Verify the repository has environments configured
3. Ensure scheduler is running (probes execute in scheduler loop)

### False "down" status

1. Verify endpoint URL is correct
2. Check if service requires authentication (liveness doesn't send auth)
3. Test with `curl` from terminal
4. Check for redirects (3xx should succeed, but verify)

### High probe latency

1. Increase interval_seconds
2. Use a lighter endpoint
3. Check network conditions
4. Consider if service is actually slow

### Notifications not working

1. Verify `notification_on_failure` is enabled
2. Check global notification settings
3. Ensure OS notification permissions granted

## Limitations

1. **HTTP only** — No TCP, gRPC, or custom protocol support
2. **No authentication** — Endpoints must be accessible without auth
3. **IPv4 only** — IPv6 endpoints may not work (check version)
4. **No certificate validation control** — Self-signed certs may fail

## See Also

- [Health](health.md) — Git repository health checks
- [Scheduler](scheduler.md) — Background automation
- [Repository](repository.md) — Environment configuration