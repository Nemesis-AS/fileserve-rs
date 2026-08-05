import { getFiles, getPublicFiles } from '$lib/services/files';
import { getUsers } from '$lib/services/users';
import { authStore } from '$lib/stores/auth.svelte';
import { DEMO_USERS } from '$lib/mock/demoAdmin';

export interface SidebarCounts {
	my: number;
	public: number;
	trash: number;
	users: number;
}

let _counts = $state<SidebarCounts>({ my: 0, public: 0, trash: 0, users: 0 });

export const countsStore = {
	get counts() {
		return _counts;
	},

	/**
	 * Re-fetch the listings the sidebar badges are counted from. Owned by a store
	 * rather than the layout so anything that changes the file set (an upload
	 * finishing, a trash/restore) can keep the badges in step without a reload.
	 *
	 * Each fetch fails independently — `/users` is admin-only, so it 403s for a
	 * regular user — and a failed one keeps its last-known count instead of
	 * blanking the badge.
	 */
	async refresh() {
		// A demo visitor is not an admin, so `/users` would 403 and leave the
		// badge on 0 beside a Users page showing four placeholder accounts.
		// Count the placeholders instead, so the two agree.
		const demo = authStore.user?.demo ?? false;

		const [files, publicFiles, users] = await Promise.all([
			getFiles().catch(() => null),
			getPublicFiles().catch(() => null),
			demo ? Promise.resolve(DEMO_USERS) : getUsers().catch(() => null)
		]);

		_counts = {
			my: files ? files.filter((f) => !f.trashed).length : _counts.my,
			public: publicFiles ? publicFiles.length : _counts.public,
			trash: files ? files.filter((f) => f.trashed).length : _counts.trash,
			users: users ? users.length : _counts.users
		};
	}
};
