# fileserve

Rust + SvelteKit fullstack file server

Backend: Rust (actix-web, sqlx + SQLite)
Frontend: SvelteKit (client/)

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

- Generate password hashes for seeding(Optional):

```sh
cargo run --bin gen_hash <password>
```
**Note:** We already have user entries in the seed script. So this step is optional.

- Paste the generated hashes into `migrations/seed.sql`, then seed the DB:

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
