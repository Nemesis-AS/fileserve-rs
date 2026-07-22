import { browser } from '$app/environment';
import type { User } from '$lib/types';
import { logout as logoutRequest } from '$lib/services/auth';

const SESSION_KEY = 'filer-session';

let _user = $state<User | null>(null);

if (browser) {
	const stored = sessionStorage.getItem(SESSION_KEY);
	if (stored) {
		try { _user = JSON.parse(stored) as User; } catch { /* ignore */ }
	}
}

export const authStore = {
	get user() { return _user; },
	get isLoggedIn() { return _user !== null; },

	login(user: User) {
		_user = user;
		if (browser) sessionStorage.setItem(SESSION_KEY, JSON.stringify(user));
	},

	updateUser(updates: Partial<User>) {
		if (!_user) return;
		_user = { ..._user, ...updates };
		if (browser) sessionStorage.setItem(SESSION_KEY, JSON.stringify(_user));
	},

	/**
	 * Clears local session state and asks the server to drop the auth cookie.
	 * Local state is cleared first and unconditionally, so callers can navigate
	 * away immediately without waiting on the request.
	 */
	logout() {
		_user = null;
		if (browser) sessionStorage.removeItem(SESSION_KEY);
		void logoutRequest();
	}
};
