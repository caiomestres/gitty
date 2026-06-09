# Troubleshooting

Common issues and their solutions.

## Installation Issues

### Windows: Smart App Control Blocks Installer

**Symptom:** Windows Smart App Control prevents running the installer.

**Solution:**

1. **Option A** — Disable Smart App Control (Developer mode):
   - Settings → Privacy & Security → Windows Security
   - App & Browser Control → Smart App Control
   - Set to "Off"

2. **Option B** — Wait for signed release:
   - SignPath.io signing in progress for releases
   - Check release notes for signing status

3. **Option C** — Build from source

### macOS: "Gitty is damaged and can't be opened"

**Symptom:** Gatekeeper blocks the app.

**Solution:**

```bash
# Remove quarantine attribute
xattr -dr com.apple.quarantine /Applications/Gitty.app

# Or via System Preferences:
# 1. Try to open Gitty (fails)
# 2. System Preferences → Security & Privacy
# 3. Click "Open Anyway"
```

### Linux: AppImage Won't Run

**Symptom:** AppImage fails to execute.

**Solution:**

```bash
# Make executable
chmod +x Gitty-x.x.x.AppImage

# If still fails, install FUSE:
# Ubuntu/Debian:
sudo apt install libfuse2

# Or extract and run:
./Gitty-x.x.x.AppImage --appimage-extract
./squashfs-root/AppRun
```

### Linux: libssl Missing

**Symptom:** Error about missing libssl.so.

**Solution:**

```bash
# Ubuntu/Debian
sudo apt install libssl3

# Older distributions may need:
sudo apt install libssl1.1
```

## Runtime Issues

### GUI Won't Start

**Symptom:** Gitty process starts but window doesn't appear.

**Diagnosis:**

```bash
# Run from terminal to see errors
gitty

# Or with logging
RUST_LOG=debug gitty
```

**Common causes:**

1. **Graphics driver issue**
   - Update GPU drivers
   - Check WebGL support

2. **Port conflict**
   - Gitty uses random high port for IPC
   - Check for port conflicts: `lsof -i :PORT`

3. **Corrupted config**
   - Reset config:
     ```bash
     mv ~/.config/gitty/config.json ~/.config/gitty/config.json.bak
     ```

4. **Missing display (headless)**
   - GUI requires display server
   - Use CLI only: `gitty-cli`

### CLI Commands Fail

**Symptom:** All commands return errors.

**Diagnosis:**

```bash
# Check binary
gitty --version
which gitty

# Check config location
gitty list --verbose
```

**Common causes:**

1. **Config permissions**
   ```bash
   ls -la ~/.config/gitty/
   # Fix permissions:
   chmod 755 ~/.config/gitty/
   chmod 644 ~/.config/gitty/config.json
   ```

2. **Config corruption**
   - Validate JSON: `python -m json.tool ~/.config/gitty/config.json`
   - Restore from backup

3. **Git not in PATH**
   ```bash
   which git
   # If missing, install Git
   ```

### Repository Shows as "Missing"

**Symptom:** Repository path no longer valid.

**Causes and solutions:**

1. **Repository moved**
   - Move back to original location, or
   - Re-scan new location (auto re-linking), or
   - Manually edit config.json (path field)

2. **External drive disconnected**
   - Reconnect drive, or
   - Unregister repository

3. **Path changed case** (Windows)
   - Windows case changes can break paths
   - Re-scan or manually update path

4. **Permissions changed**
   - Fix permissions on directory
   - Or run Gitty with appropriate privileges

### Scan Finds No Repositories

**Symptom:** Scan completes but finds 0 repos.

**Diagnosis:**

```bash
# Verify path
gitty scan /path/to/scan --verbose

# Check for .git directories manually
find /path/to/scan -name ".git" -type d
```

**Common causes:**

1. **No .git directories**
   - Path doesn't contain Git repositories
   - Verify with `find` command above

2. **Bare repositories**
   - Gitty doesn't support bare repos
   - Use cloned repositories with working directories

3. **Permission denied**
   - Gitty can't read the directory
   - Check permissions on path

4. **Symlinks**
   - Gitty doesn't follow symlinks into repos
   - Use actual paths in scan roots

### Fetch/Pull Failures

**Symptom:** Git operations fail on some repositories.

**Diagnosis:**

```bash
# Try manually
cd /path/to/repo
git fetch

# Check remote
git remote -v
git ls-remote origin
```

**Common causes:**

1. **Authentication failure**
   - SSH key not loaded: `ssh-add -l`
   - HTTPS credentials expired
   - 2FA required

2. **Network issue**
   - Firewall blocking Git
   - Corporate proxy
   - DNS resolution failing

3. **Remote removed**
   - Repository deleted from GitHub/GitLab
   - Remote URL changed

4. **Merge conflicts** (pull only)
   - Uncommitted local changes
   - Diverged branches

**Solutions:**

```bash
# For auth issues, test SSH:
ssh -T git@github.com

# For HTTPS, check credential helper:
git config credential.helper

# For merge conflicts, resolve manually:
git status
git merge --abort  # To cancel
```

### Scheduler Not Running

**Symptom:** Scheduler enabled but no activity logged.

**GUI mode:**

1. Check GUI hasn't been backgrounded
2. Verify app not suspended (macOS App Nap)
3. Check logs for scheduler errors

**CLI daemon:**

```bash
# Check status
gitty scheduler status

# Try foreground mode to see errors
gitty scheduler start --foreground

# Check PID file
ls -la ~/.config/gitty/scheduler.pid
```

**Common causes:**

1. **Process killed**
   - Check system logs
   - OOM killer (out of memory)

2. **Power policy blocking**
   - Laptop on battery with `AcOnly` policy
   - Low battery with threshold policy

3. **Time window**
   - Current time outside configured window
   - Wrong day of week

4. **PID file stale**
   ```bash
   # Remove stale PID file
   rm ~/.config/gitty/scheduler.pid
   gitty scheduler start
   ```

### Health Evaluation Fails

**Symptom:** Health view shows old data or errors.

**Diagnosis:**

```bash
# Try manual evaluation
gitty health --verbose

# Check specific repo
gitty health --repo myapp
```

**Common causes:**

1. **Repository missing**
   - Missing repos excluded from health
   - Fix path or unregister

2. **Git lock files**
   - Another Git process running
   - Remove stale locks:
     ```bash
     rm /path/to/repo/.git/index.lock
     ```

3. **Permission denied**
   - Can't read Git metadata
   - Fix repository permissions

### Performance Issues

**Symptom:** Slow UI or operations.

**GUI:**

1. **Reduce items per page**
   - Dashboard pagination: 25 items
   - Close unused tabs

2. **Check for large repos**
   - Repos with 100k+ commits
   - Many untracked files
   - Large working directories

3. **Disable liveness** (if not needed)
   - Each probe consumes resources
   - Reduce probe frequency

**CLI:**

1. **Use filters**
   ```bash
   # Instead of:
   gitty fetch
   # Try:
   gitty fetch --group active
   ```

2. **Parallel limits**
   - Gitty limits parallel Git operations
   - Sequential may be faster for many repos

## Error Messages

### "Lock file exists"

Another Gitty process is operating on the repository. Wait or check for stale locks.

### "Repository not found"

UUID doesn't exist in config. Check with `gitty list`.

### "Group not found"

Group path doesn't exist. Create with `gitty group create`.

### "Macro not found"

Macro name doesn't exist. Check with `gitty macro list`.

### "Invalid UUID format"

UUID should be: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`

## Getting Help

### Check Logs

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\gitty\logs\` |
| macOS | `~/Library/Application Support/gitty/logs/` |
| Linux | `~/.config/gitty/logs/` |

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug
gitty

# Or for specific modules
export RUST_LOG=gitty_core=debug
gitty
```

### Issue Report

When reporting issues, include:

1. **Gitty version:** `gitty --version`
2. **Operating system:** macOS 14.2, Windows 11, Ubuntu 22.04
3. **Repro steps:** Exact commands or UI actions
4. **Expected vs actual:** What should happen vs what happens
5. **Logs:** Relevant log excerpts
6. **Config:** `config.json` (redact sensitive paths if needed)

Report at: https://github.com/caiomestres/gitty/issues

## Recovery Procedures

### Reset Configuration

```bash
# Backup first
cp ~/.config/gitty/config.json ~/.config/gitty/config.json.bak

# Reset (delete config)
rm ~/.config/gitty/config.json

# Restart Gitty — creates fresh config
gitty
```

### Rebuild from Scratch

```bash
# 1. Backup
mkdir ~/gitty-backup
cp -r ~/.config/gitty/* ~/gitty-backup/

# 2. Reset
rm -rf ~/.config/gitty/*

# 3. Reinitialize
gitty  # Creates fresh config

# 4. Re-add scan roots
gitty scan ~/projects

# 5. Reconfigure settings via GUI
```

### Fix Stale Lock

```bash
# Find and remove stale lock files
find ~/.config/gitty/locks/ -name "*.lock" -delete

# Or specific repo lock
rm ~/.config/gitty/locks/<repo-uuid>.lock
```

## See Also

- [FAQ](../intro/faq.md) — Common questions
- [Development](development.md) — Building from source
- [GitHub Issues](https://github.com/caiomestres/gitty/issues) — Bug reports