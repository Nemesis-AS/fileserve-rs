-- DEVELOPMENT FIXTURE ONLY. NEVER RUN THIS ON A PUBLIC HOST.
--
-- The two passwords below are written in the clear in this file, which is
-- published in the repository, so anyone can sign in as either account. On a
-- demo or any other internet-facing deployment that hands out an admin session
-- to the world. Real deployments get their first account from `seed_admin`,
-- which generates a password on first boot and prints it once.

INSERT INTO users (username, name, password, role) VALUES
    ('admin', 'Administrator', '$2b$10$sciGMGoAHPlXBRKGaQ0utOKZQ2Ul1ogNXmzfpdGjhi6wA6mIM9cwy', 'admin'), -- Admin@123
    ('user', 'Regular User', '$2b$10$CV1ybDlxgwxX57hd/uo//OTeJK9.XPFD4f3djUBe4gIF.ucmYcKZO', 'user'); -- User@123
