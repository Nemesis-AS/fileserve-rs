import { browser } from '$app/environment';
import { goto } from '$app/navigation';
import { authStore } from '$lib/stores/auth.svelte';

let handling = false;

/**
 * Invoked when an *authenticated* request returns 401 — the session has expired
 * or been revoked server-side. Clears local auth and sends the user back to
 * sign in.
 *
 * The `handling` guard collapses a burst of concurrent 401s (e.g. the parallel
 * sidebar fetches) into a single logout + redirect, then resets once navigation
 * settles so a later expiry after re-login is handled again.
 *
 * Deliberately NOT wired into the login endpoint: a 401 there means "wrong
 * password", a form error the login page surfaces itself — not a lost session.
 */
export function handleUnauthorized(): void {
	if (!browser || handling) return;
	handling = true;
	authStore.logout();
	void goto('/login', { replaceState: true }).finally(() => {
		handling = false;
	});
}
