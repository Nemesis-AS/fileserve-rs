-- Persistent per-file visibility. `public = 1` means any authenticated user can
-- list and download the file; sharing was previously only via time-limited JWT
-- share links, which stay as the mechanism for unauthenticated external access.
ALTER TABLE files ADD COLUMN public BOOLEAN NOT NULL DEFAULT 0;

-- Backs the cross-owner `GET /files/public` listing.
CREATE INDEX idx_files_public ON files(public);
