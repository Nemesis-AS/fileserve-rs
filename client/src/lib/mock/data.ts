import type { FilerFile, User, AuditEvent } from '$lib/types';
import { extOf, fileCategory, fileColor } from '$lib/utils/file';

const NOW = Date.now();
const D = (days: number, hours = 0) =>
	new Date(NOW - days * 86_400_000 - hours * 3_600_000).toISOString();

const THUMBS: Record<string, string | null> = {
	'mountain-lake.jpg': 'https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=400&q=70',
	'workspace.jpg': 'https://images.unsplash.com/photo-1499951360447-b19be8fe80f5?w=400&q=70',
	'cabin-snow.jpeg': 'https://images.unsplash.com/photo-1551582045-6ec9c11d8697?w=400&q=70',
	'rover-test.png': 'https://images.unsplash.com/photo-1614642264762-d0a3b8bf3700?w=400&q=70',
	'sunset-roof.heic': 'https://images.unsplash.com/photo-1502082553048-f009c37129b9?w=400&q=70',
	'logo-final.svg': null
};

const _raw: Omit<FilerFile, 'id' | 'ext' | 'category' | 'color' | 'thumb'>[] = [
	{ name: 'mountain-lake.jpg', size: 3_412_005, modified: D(0, 2), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'workspace.jpg', size: 1_882_312, modified: D(0, 6), owner: 'me', public: true, trashed: false, trashedAt: null },
	{ name: 'cabin-snow.jpeg', size: 2_511_843, modified: D(2, 5), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'rover-test.png', size: 742_188, modified: D(4), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'sunset-roof.heic', size: 4_120_550, modified: D(6), owner: 'me', public: true, trashed: false, trashedAt: null },
	{ name: 'logo-final.svg', size: 38_412, modified: D(9), owner: 'me', public: true, trashed: false, trashedAt: null },
	{ name: 'lease-2026.pdf', size: 1_234_567, modified: D(1), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'invoice-Q1.pdf', size: 312_408, modified: D(3), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'reading-notes.md', size: 22_104, modified: D(0, 1), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'recipe-shakshuka.md', size: 8_312, modified: D(11), owner: 'me', public: true, trashed: false, trashedAt: null },
	{ name: 'travel-itinerary.docx', size: 142_882, modified: D(7), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'tax-return-2025.pdf', size: 2_120_004, modified: D(28), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'home-server.tf', size: 14_882, modified: D(5), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'sync.sh', size: 4_120, modified: D(13), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'nginx.conf', size: 6_888, modified: D(16), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'backup.py', size: 12_344, modified: D(21), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'index.html', size: 22_011, modified: D(34), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'energy-bill-2025.csv', size: 102_400, modified: D(8), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'addresses.json', size: 18_220, modified: D(12), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'household-budget.xlsx', size: 402_211, modified: D(2, 8), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'birthday-2025.mp4', size: 412_001_233, modified: D(14), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'piano-loop-Cmaj.flac', size: 24_882_001, modified: D(18), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'voicememo-2026-05-12.m4a', size: 2_188_443, modified: D(3, 2), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'family-photos-2024.zip', size: 1_840_001_100, modified: D(45), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'old-laptop-backup.tar.gz', size: 8_120_443_882, modified: D(60), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'router-syslog.txt', size: 234_000, modified: D(0, 4), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'pi-temperature.log', size: 88_300, modified: D(1, 1), owner: 'me', public: false, trashed: false, trashedAt: null },
	{ name: 'README.txt', size: 912, modified: D(95), owner: 'me', public: true, trashed: false, trashedAt: null },
	{ name: 'movie-night-poster.png', size: 902_344, modified: D(1, 4), owner: 'sam', public: true, trashed: false, trashedAt: null },
	{ name: 'shared-album-link.txt', size: 180, modified: D(2, 9), owner: 'leo', public: true, trashed: false, trashedAt: null }
];

let _id = 1;

export const MOCK_FILES: FilerFile[] = [
	..._raw.map((f) => ({
		id: 'f' + _id++,
		ext: extOf(f.name),
		category: fileCategory(f.name),
		color: fileColor(f.name),
		thumb: THUMBS[f.name] ?? null,
		...f
	})),
	{
		id: 'f' + _id++, name: 'draft-letter.md', ext: 'md', category: fileCategory('x.md'),
		color: fileColor('x.md'), thumb: null, size: 6020, modified: D(4, 6),
		owner: 'me', public: false, trashed: true, trashedAt: D(0, 3)
	},
	{
		id: 'f' + _id++, name: 'scratch.txt', ext: 'txt', category: fileCategory('x.txt'),
		color: fileColor('x.txt'), thumb: null, size: 412, modified: D(7),
		owner: 'me', public: false, trashed: true, trashedAt: D(1, 6)
	}
];

export const MOCK_USERS: User[] = [
	{ id: 'u1', name: 'Alex Chen', username: 'alex', email: 'alex@home.lan', role: 'admin', status: 'active', quotaGB: 200, usedGB: 84.2, lastSeen: D(0, 0.1), files: 28 },
	{ id: 'u2', name: 'Sam Reyes', username: 'sam', email: 'sam@home.lan', role: 'user', status: 'active', quotaGB: 50, usedGB: 12.4, lastSeen: D(0, 6), files: 14 },
	{ id: 'u3', name: 'Leo Park', username: 'leo', email: 'leo@home.lan', role: 'user', status: 'active', quotaGB: 50, usedGB: 31.8, lastSeen: D(1, 2), files: 41 },
	{ id: 'u4', name: 'Mira Patel', username: 'mira', email: 'mira@home.lan', role: 'user', status: 'active', quotaGB: 50, usedGB: 4.2, lastSeen: D(3), files: 6 },
	{ id: 'u5', name: 'Jules Kim', username: 'jules', email: 'jules@home.lan', role: 'user', status: 'suspended', quotaGB: 50, usedGB: 8.1, lastSeen: D(22), files: 11 }
];

const AT = (mins: number) => new Date(NOW - mins * 60_000).toISOString();

export const MOCK_AUDIT: AuditEvent[] = [
	{ id: 'e60', at: AT(2), type: 'upload', actor: 'alex', target: 'reading-notes.md', ip: '10.0.1.42', meta: '22 KB' },
	{ id: 'e59', at: AT(8), type: 'download', actor: 'sam', target: 'mountain-lake.jpg', ip: '10.0.1.43', meta: 'shared link' },
	{ id: 'e58', at: AT(14), type: 'login', actor: 'alex', target: null, ip: '10.0.1.42', meta: 'web' },
	{ id: 'e57', at: AT(33), type: 'rename', actor: 'alex', target: 'travel-itinerary.docx', ip: '10.0.1.42', meta: 'from "trip-may.docx"' },
	{ id: 'e56', at: AT(40), type: 'share', actor: 'alex', target: 'workspace.jpg', ip: '10.0.1.42', meta: 'made public' },
	{ id: 'e55', at: AT(58), type: 'delete', actor: 'alex', target: 'draft-letter.md', ip: '10.0.1.42', meta: 'to Trash' },
	{ id: 'e54', at: AT(74), type: 'upload', actor: 'leo', target: 'shared-album-link.txt', ip: '10.0.1.44', meta: '180 B' },
	{ id: 'e53', at: AT(91), type: 'login', actor: 'leo', target: null, ip: '10.0.1.44', meta: 'web' },
	{ id: 'e52', at: AT(120), type: 'download', actor: 'alex', target: 'home-server.tf', ip: '10.0.1.42', meta: null },
	{ id: 'e51', at: AT(180), type: 'upload', actor: 'alex', target: 'birthday-2025.mp4', ip: '10.0.1.42', meta: '412 MB' },
	{ id: 'e50', at: AT(245), type: 'auth_fail', actor: 'jules', target: null, ip: '185.243.218.50', meta: 'wrong password' },
	{ id: 'e49', at: AT(248), type: 'auth_fail', actor: 'jules', target: null, ip: '185.243.218.50', meta: 'wrong password' },
	{ id: 'e48', at: AT(252), type: 'auth_fail', actor: 'jules', target: null, ip: '185.243.218.50', meta: 'wrong password' },
	{ id: 'e47', at: AT(290), type: 'user_edit', actor: 'alex', target: '@jules', ip: '10.0.1.42', meta: 'suspended' },
	{ id: 'e46', at: AT(310), type: 'share', actor: 'alex', target: 'movie-night-poster.png', ip: '10.0.1.42', meta: 'made public' },
	{ id: 'e45', at: AT(340), type: 'download', actor: 'mira', target: 'recipe-shakshuka.md', ip: '10.0.1.45', meta: 'shared link' },
	{ id: 'e44', at: AT(420), type: 'login', actor: 'sam', target: null, ip: '10.0.1.43', meta: 'web' },
	{ id: 'e43', at: AT(540), type: 'restore', actor: 'alex', target: 'old-notes.txt', ip: '10.0.1.42', meta: 'from Trash' },
	{ id: 'e42', at: AT(720), type: 'user_add', actor: 'alex', target: '@mira', ip: '10.0.1.42', meta: 'role: user' },
	{ id: 'e41', at: AT(840), type: 'download', actor: 'alex', target: 'family-photos-2024.zip', ip: '10.0.1.42', meta: null },
	{ id: 'e40', at: AT(1020), type: 'login', actor: 'alex', target: null, ip: '10.0.1.42', meta: 'web' },
	{ id: 'e39', at: AT(1260), type: 'upload', actor: 'alex', target: 'household-budget.xlsx', ip: '10.0.1.42', meta: '402 KB' },
	{ id: 'e38', at: AT(1380), type: 'delete', actor: 'leo', target: 'temp-export.csv', ip: '10.0.1.44', meta: 'to Trash' },
	{ id: 'e37', at: AT(1620), type: 'password', actor: 'sam', target: null, ip: '10.0.1.43', meta: 'changed by user' },
	{ id: 'e36', at: AT(2200), type: 'login', actor: 'mira', target: null, ip: '10.0.1.45', meta: 'web' },
	{ id: 'e35', at: AT(2880), type: 'share', actor: 'leo', target: 'shared-album-link.txt', ip: '10.0.1.44', meta: 'made public' }
];

export const TEXT_PREVIEWS: Record<string, string> = {
	'reading-notes.md': `# Reading notes — May\n\n## Power-Knowledge, Foucault\n- Discipline isn't the opposite of freedom; it's a *productive* mode of power.\n- The panopticon as a diagram, not a building.\n- "Visibility is a trap." — page 200ish\n\n## Notes on a homelab\n- Move primary backups off-site after we're done with the Q2 cleanup.\n- Replace the old WD drive in slot 3. Smartmontools says SMART codes 5, 197, 198.\n- Tighten nginx rate-limit on /upload.\n- Test restore from cold storage at least once before fall.\n\n## Books queue\n1. The Mushroom at the End of the World\n2. Index, A History of The\n3. Stoner — re-read\n`,
	'sync.sh': `#!/usr/bin/env bash\n# Pull-from-remote backup, runs every 6h via systemd timer.\n\nset -euo pipefail\n\nSRC="rsync://nas.home.lan/archive/"\nDST="/mnt/cold/archive/"\nLOG="/var/log/sync.log"\n\ndate >> "$LOG"\nrsync -avz --delete --human-readable "$SRC" "$DST" >> "$LOG" 2>&1\necho "ok" >> "$LOG"\n`,
	'nginx.conf': `# /etc/nginx/sites-enabled/files\nserver {\n  listen 443 ssl http2;\n  server_name files.home.lan;\n\n  client_max_body_size 5G;\n  ssl_certificate     /etc/letsencrypt/live/files.home.lan/fullchain.pem;\n  ssl_certificate_key /etc/letsencrypt/live/files.home.lan/privkey.pem;\n\n  location / {\n    proxy_pass http://127.0.0.1:8080;\n    proxy_set_header Host $host;\n    proxy_set_header X-Real-IP $remote_addr;\n  }\n}\n`,
	'router-syslog.txt': `May 15 09:14:22 router kernel: [12345.678] eth0: link up\nMay 15 09:14:23 router dhcpd: DHCPACK on 10.0.1.42\nMay 15 09:14:25 router dnsmasq: query[A] github.com from 10.0.1.42\nMay 15 09:15:11 router kernel: [12394.001] DROP IN=eth1 SRC=185.243.218.50 DPT=22\n`,
	'README.txt': `home.lan — file server\n======================\n\nThis is the file pool for the home server. It's small on purpose.\n\nConventions\n-----------\n- Names are kebab-case. Dates as YYYY-MM-DD.\n- Anything in /public is link-shareable.\n- Anything in /trash is purged after 30 days.\n\nAdmin\n-----\n- alex (that's me)\n- ssh root@home.lan for serial console\n\nBackups go to the cold drive every 6h. See sync.sh.\n`,
	'addresses.json': `{\n  "alex":   { "addr": "10.0.1.42", "mac": "a4:5e:60:11:22:33" },\n  "sam":    { "addr": "10.0.1.43", "mac": "b8:27:eb:aa:bb:cc" },\n  "leo":    { "addr": "10.0.1.44", "mac": "c0:ee:fb:dd:ee:ff" },\n  "mira":   { "addr": "10.0.1.45", "mac": "dc:a6:32:99:88:77" },\n  "router": { "addr": "10.0.1.1",  "mac": "e0:9d:31:55:66:77" }\n}\n`
};
