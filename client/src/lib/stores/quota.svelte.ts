import { getQuota, type Quota } from '$lib/services/users';

let _quota = $state<Quota>({ quotaGB: null, usedGB: 0, files: 0 });

export const quotaStore = {
	get quota() {
		return _quota;
	},

	/**
	 * Re-fetch quota + usage from the server. Failures are swallowed so the
	 * last-known values stay on screen; the bar is informational, not a gate.
	 */
	async refresh() {
		try {
			_quota = await getQuota();
		} catch {
			/* keep last-known values */
		}
	}
};
