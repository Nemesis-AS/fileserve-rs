# fileserve

Rust + SvelteKit fullstack file server

Backend: Rust (actix-web, sqlx + SQLite)
Frontend: SvelteKit (client/)

### To Do

- [x] User creation routes
- [x] Trash
- [x] Storage Quota
- [x] Password Change
- [x] Self-update
- [ ] Audit Log
- [ ] Signed releases (see "Security" under Updates)

### Prerequisites

- Rust toolchain (stable)
- Node.js (for frontend)
- `sqlx-cli` for SQLite:

```sh
cargo install sqlx-cli --no-default-features --features sqlite
```

### Setup

1. Clone the repository.
2. Install frontend dependencies:

```sh
cd client
npm install
```

### Database

- Create the database file:

```sh
mkdir -p data
touch data/db.sqlite3
```
OR on Windows:
```ps
mkdir data
New-Item data/db.sqlite3 -ItemType File
```

- Run migrations:

```sh
sqlx migrate run --database-url sqlite:data/db.sqlite3
```

- Then seed the DB:

```sh
sqlite3 data/db.sqlite3 < scripts/seed.sql
```
OR on Windows:

```ps
type migrations/seed.sql | sqlite3 data/db.sqlite3
```

### Running

- Backend (default port 8112):

```sh
cargo run
```

- Frontend (from repository root):

```sh
cd client
npm run dev
```

Backend listens on port 8112 by default.

## Updates

The server checks its own GitHub releases on startup and every 24 hours. When a
newer version exists, **Admin → Configuration → Updates** offers to install it:
the release asset for the running platform is downloaded, checked against the
release's `SHA256SUMS`, and swapped in. Nothing is downloaded or installed
without an admin clicking for it.

Installing does not take effect until the server restarts. The same panel has a
**Restart now** button.

### Rollback

Updates are reversible without intervention:

- The previous binary is kept as `fileserve-rs.exe.old` and a record is written
  to `data/update-pending.json` before anything is replaced.
- After restarting, the old process waits for the new one to answer `/health`.
  If it doesn't within 30 seconds, or exits first, the previous binary is put
  back and started again.
- If the new binary starts but crashes before it has been up for 10 seconds, a
  boot counter in that record trips on the next start and rolls back too. This
  is the path that matters under a service manager, where the check above can't
  run.
- Once a new version has been up for 10 seconds it is confirmed: the record and
  the old binary are deleted.

If both nets fail, recovery is manual — rename `fileserve-rs.exe.old` (or
`.failed`) back over `fileserve-rs.exe`.

### Running under a service manager

Under systemd or Docker a process that spawns a replacement and exits gets that
replacement killed with it, so the server instead **exits** and relies on the
supervisor to start the new binary. This is detected automatically, and can be
forced either way with `UPDATE_RESTART_MODE`.

**The unit must have a restart policy**, or "Restart now" is a stop button:

```ini
[Service]
Restart=always
```

```sh
docker run --restart unless-stopped ...
```

### Security

The download is fetched over HTTPS and verified against a SHA256 published as a
release asset. That protects against a corrupted download or a hostile proxy —
it is **not** a signature. The checksum comes from the same release as the
binary, so anyone able to publish a release to this repository can publish a
matching pair, and the updater will install it and run it with the server's
privileges. Enable 2FA and branch protection on the repository accordingly;
signed releases are on the To Do list above.

Set `SELF_UPDATE_ENABLED=false` where the binary is package-managed or
root-owned — the server should not be rewriting a binary it doesn't own. Prefer
running as an unprivileged user that owns its own executable.

TLS root certificates are compiled in, so a binary left unupdated for years may
eventually fail to reach GitHub; download that release manually.

See `.env.example` for `UPDATE_REPO`, `UPDATE_API_BASE`, `SELF_UPDATE_ENABLED`
and `UPDATE_RESTART_MODE`.

## Releasing

`.github/workflows/release.yml` builds and publishes on a tag. Asset names and
the `SHA256SUMS` format are a contract with the updater in `src/update/` — if
you publish releases by hand, match them exactly or existing installs will not
find their download.

1. Bump `version` in `Cargo.toml`.
2. Commit, then tag and push:

```sh
git tag v0.1.2
git push origin v0.1.2
```

The workflow fails fast if the tag and `Cargo.toml` disagree — a binary that
reports a different version than its release tag would offer to update to
itself forever. It produces:

```
fileserve-rs-v0.1.2-x86_64-pc-windows-msvc.exe
fileserve-rs-v0.1.2-x86_64-unknown-linux-gnu
SHA256SUMS
```

Cut the first tag of any risky change as a **pre-release** — `/releases/latest`
excludes them, so a broken build reaches nobody.
