import type { User } from '$lib/types';

const API = '/api/v1';

/** Envelope every backend endpoint wraps its payload in. */
interface ApiEnvelope<T> {
	success: boolean;
	message: string | null;
	data: T | null;
}

export interface LoginResult {
	user: User;
	token: string;
}

async function unwrap<T>(res: Response, fallbackMsg: string): Promise<T> {
	let body: ApiEnvelope<T> | null = null;
	try {
		body = (await res.json()) as ApiEnvelope<T>;
	} catch {
		// Non-JSON response (proxy error, server down) fall through.
	}

	if (!res.ok || !body?.success) {
		throw new Error(body?.message ?? fallbackMsg);
	}
	if (body.data === null || body.data === undefined) {
		throw new Error(fallbackMsg);
	}
	return body.data;
}

export async function login(username: string, password: string): Promise<LoginResult> {
	const res = await fetch(`${API}/auth/login`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		credentials: 'include',
		body: JSON.stringify({ username, password })
	});
	return unwrap<LoginResult>(res, 'Sign in failed');
}

/**
 * Clears the server session cookie. Never throws: the caller always clears local
 * state, so a failure here must not strand the user in a signed-in UI.
 */
export async function logout(): Promise<void> {
	try {
		await fetch(`${API}/auth/logout`, { method: 'POST', credentials: 'include' });
	} catch {
		// Network failure — local session is cleared by the caller regardless.
	}
}
