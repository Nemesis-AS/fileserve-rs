import type { ServerConfig } from '$lib/types';

const API = '/api/v1';

/**
 * Fetches the anonymous boot config.
 *
 * Deliberately does not go through `apiFetch`: that routes a 401 into
 * `handleUnauthorized()`, and this endpoint is meant to be callable with no
 * session at all, including from the login page.
 */
export async function getServerConfig(): Promise<ServerConfig> {
	const res = await fetch(`${API}/config`);
	const body = await res.json();
	if (!res.ok || !body?.success || !body.data) {
		throw new Error(body?.message ?? 'Could not load server config');
	}
	return body.data as ServerConfig;
}
