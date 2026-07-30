# Installing fileserve.rs

## Running a release

Most people don't need to build anything. Grab the latest binary for your platform from the [releases page](https://github.com/Nemesis-AS/fileserve-rs/releases/latest) and run it.

```sh
./fileserve-rs-vX.Y.Z-x86_64-unknown-linux-gnu
```

On Windows, just double-click the `.exe`, or run it from a terminal:

```ps
.\fileserve-rs-vX.Y.Z-x86_64-pc-windows-msvc.exe
```

On first run it creates a `data/` folder for its database and a `files/` folder for uploads, right next to the binary, generates a JWT secret, and creates an admin account for you. If you didn't set `ADMIN_USERNAME` and `ADMIN_PASSWORD` beforehand, a generated password is printed once to the console, so keep an eye on the output the first time you start it.

The server listens on port 8112. Open `http://localhost:8112` (or the machine's address if you're running it remotely) and log in.

Want to set the storage location, upload size limit, or admin credentials up front instead of after the fact? Set `STORAGE_PATH`, `TUS_MAX_SIZE`, `ADMIN_USERNAME`, or `ADMIN_PASSWORD` as environment variables before starting it. See `.env.example` for the full list. Everything here can also be changed later from the admin Configuration page.

If you're running this long-term rather than just trying it out, see [Running under a service manager](#running-under-a-service-manager) below.

## Building from source

For contributors, or if there's no prebuilt binary for your platform.

### Prerequisites

- Rust toolchain (stable)
- Node.js (for the frontend)
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

The server creates and migrates its own database automatically on first boot, so this step is only needed if you're changing the schema and want `sqlx-cli` to check queries against a live database, or if you want to load the sample data used for local development.

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

- Then seed the DB with sample data:

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

## Running under a service manager

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
