#!/usr/bin/env bash
#
# Sets up a fake GitHub release so the self-updater can be exercised without
# publishing anything. Run from the repository root.
#
#   scripts/update-fixture.sh          # a healthy newer version
#   scripts/update-fixture.sh broken   # a newer version that won't start
#
# The "broken" mode is the one worth running: it proves the server notices the
# replacement failing to come up and puts the previous binary back on its own.
#
# Builds two binaries — the current version, and the next patch version standing
# in as the release — then stages them where a local web server can serve them
# in the shape the GitHub API returns. Cargo.toml is restored on exit.

set -euo pipefail

MODE="${1:-good}"
FIXTURE_PORT=9000
REPO="fixture/fileserve-rs"

if [ ! -f Cargo.toml ] || [ ! -d src/update ]; then
	echo "Run this from the repository root." >&2
	exit 1
fi

case "$MODE" in
	good | broken) ;;
	*)
		echo "Usage: $0 [good|broken]" >&2
		exit 1
		;;
esac

ROOT="$(pwd)"
WORK="$ROOT/tmp/update-fixture"
FIXTURES="$WORK/fixtures"
SANDBOX="$WORK/sandbox"

CURRENT="$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)"
NEXT="$(echo "$CURRENT" | awk -F. '{ printf "%s.%s.%d", $1, $2, $3 + 1 }')"

# Whatever happens, don't leave the crate on the wrong version.
restore_version() {
	sed -i "s/^version = \".*\"/version = \"$CURRENT\"/" "$ROOT/Cargo.toml"
}
trap restore_version EXIT

if [ "$(uname -o 2>/dev/null)" = "Msys" ] || [ "${OS:-}" = "Windows_NT" ]; then
	EXE=".exe"
	TARGET="x86_64-pc-windows-msvc"
	# Spawns, exits immediately, never serves anything — exactly the failure
	# the startup watchdog exists to catch.
	BROKEN_SRC="/c/Windows/System32/whoami.exe"
else
	EXE=""
	TARGET="x86_64-unknown-linux-gnu"
	BROKEN_SRC="/bin/true"
fi

ASSET="fileserve-rs-v$NEXT-$TARGET$EXE"

rm -rf "$WORK"
mkdir -p "$FIXTURES/repos/$REPO/releases" "$SANDBOX"

echo "==> Building v$NEXT (stands in as the release)"
sed -i "s/^version = \".*\"/version = \"$NEXT\"/" Cargo.toml
cargo build --quiet
if [ "$MODE" = "broken" ]; then
	echo "    using a deliberately broken binary for v$NEXT"
	cp "$BROKEN_SRC" "$FIXTURES/$ASSET"
else
	cp "target/debug/fileserve-rs$EXE" "$FIXTURES/$ASSET"
fi

echo "==> Rebuilding v$CURRENT (the server you'll run)"
restore_version
cargo build --quiet
cp "target/debug/fileserve-rs$EXE" "$SANDBOX/"

echo "==> Staging the release"
(cd "$FIXTURES" && sha256sum "$ASSET" > SHA256SUMS)
SIZE="$(wc -c < "$FIXTURES/$ASSET" | tr -d ' ')"

cat > "$FIXTURES/repos/$REPO/releases/latest" <<JSON
{
  "tag_name": "v$NEXT",
  "name": "$NEXT — fixture release",
  "body": "Served by scripts/update-fixture.sh.\n\n* Mode: $MODE\n* Not a real release.",
  "html_url": "http://127.0.0.1:$FIXTURE_PORT/repos/$REPO/releases/latest",
  "published_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "draft": false,
  "prerelease": false,
  "assets": [
    {
      "name": "$ASSET",
      "browser_download_url": "http://127.0.0.1:$FIXTURE_PORT/$ASSET",
      "size": $SIZE
    },
    {
      "name": "SHA256SUMS",
      "browser_download_url": "http://127.0.0.1:$FIXTURE_PORT/SHA256SUMS",
      "size": $(wc -c < "$FIXTURES/SHA256SUMS" | tr -d ' ')
    }
  ]
}
JSON

cat <<INSTRUCTIONS

Ready. v$CURRENT will be offered an update to v$NEXT ($MODE).

  1. Serve the fixture release — leave this running:

       (cd tmp/update-fixture/fixtures && python -m http.server $FIXTURE_PORT --bind 127.0.0.1)

  2. Start the server in a second terminal, so you can watch what it prints:

       cd tmp/update-fixture/sandbox
       ADMIN_PASSWORD=testpass123 \\
         UPDATE_API_BASE=http://127.0.0.1:$FIXTURE_PORT \\
         UPDATE_REPO=$REPO \\
         ./fileserve-rs$EXE

  3. Open http://localhost:8112, sign in as admin / testpass123, and go to
     Admin -> Configuration. The Updates section should offer v$NEXT.
     Click "Download & install", then "Restart now".

INSTRUCTIONS

if [ "$MODE" = "broken" ]; then
	cat <<'BROKEN'
  Expect: the restart fails, the console reports that the new version exited
  immediately, the previous binary is put back and started again, and the page
  reports the rollback. The server keeps serving on the old version.

BROKEN
else
	cat <<GOOD
  Expect: the page waits for the server, then reports it restarted on v$NEXT.
  About ten seconds later the console confirms the update and deletes
  fileserve-rs$EXE.old. Check the version with:

       curl http://localhost:8112/api/v1/health

  Everything lives in tmp/update-fixture/ — delete it when you're done.

GOOD
fi

echo "  To test the checksum check, corrupt a digit and retry the install:"
echo "       sed -i 's/^./f/' tmp/update-fixture/fixtures/SHA256SUMS"
echo ""
