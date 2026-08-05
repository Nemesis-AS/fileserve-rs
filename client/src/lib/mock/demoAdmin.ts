import type { User } from '$lib/types';
import type { UpdateStatus } from '$lib/services/updates';

/**
 * Placeholder data for the admin screens on a demo deployment.
 *
 * Rendered client-side rather than served by the API. That is the whole point:
 * a demo visitor's browser never calls the admin endpoints at all, so there is
 * no code path on the server that could hand out a real user list, the real
 * storage path, or a live count of other visitors. The server simply refuses
 * those routes, as it does for any non-admin.
 *
 * The names match the actors in `$lib/mock/data.ts`, so the Users page and the
 * Audit log tell one consistent story.
 */
export const DEMO_USERS: User[] = [
	{
		id: 'alex',
		username: 'alex',
		name: 'Alex Rivera',
		role: 'admin',
		status: 'active',
		quotaGB: 100,
		usedGB: 41.2,
		files: 318
	},
	{
		id: 'sam',
		username: 'sam',
		name: 'Sam Okafor',
		role: 'user',
		status: 'active',
		quotaGB: 20,
		usedGB: 12.6,
		files: 154
	},
	{
		id: 'leo',
		username: 'leo',
		name: 'Leo Fontaine',
		role: 'user',
		status: 'active',
		quotaGB: 20,
		usedGB: 3.1,
		files: 47
	},
	{
		id: 'mira',
		username: 'mira',
		name: 'Mira Haddad',
		role: 'user',
		status: 'suspended',
		quotaGB: 20,
		usedGB: 0,
		files: 0
	}
];

/**
 * Cosmetic settings for the demo's read-only Configuration page.
 *
 * These are the product's own defaults, not the demo's actual caps. The point
 * of the page is to show what an admin sees on a real deployment, and the
 * demo's real 25 MB ceiling rendered in this GB-denominated field would read
 * "0.024", which looks like a bug rather than a setting. The visitor's true
 * upload limit is surfaced where it matters, on the upload control itself.
 */
export const DEMO_SETTINGS = {
	storagePath: '/srv/fileserve/files',
	maxUploadGB: 5,
	defaultQuotaGB: 20
};

/**
 * Canned update status. `enabled: false` is not a fiction for the demo's sake:
 * a demo host really does run with `SELF_UPDATE_ENABLED=false`, and the page
 * already renders a proper "self-update is disabled" panel for that state, so
 * this needs no extra UI.
 */
export const DEMO_UPDATE_STATUS: UpdateStatus = {
	currentVersion: '0.1.2',
	target: 'x86_64-unknown-linux-gnu',
	enabled: false,
	lastChecked: null,
	checking: false,
	checkError: null,
	latest: null,
	install: { phase: 'idle', downloadedBytes: 0, totalBytes: 0, version: null, error: null }
};
