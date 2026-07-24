import { handleUnauthorized } from './session';

const API = '/api/v1';

interface ApiEnvelope<T> {
	success: boolean;
	message: string | null;
	data: T | null;
}

/**
 * Calls a `/api/v1` endpoint and unwraps the `{ success, message, data }`
 * envelope, surfacing the server's `message` as the thrown error. No mock
 * fallback: these are authenticated mutations, so failures must be visible.
 */
export async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
	const res = await fetch(`${API}${path}`, { credentials: 'include', ...init });

	if (res.status === 401) handleUnauthorized();

	let body: ApiEnvelope<T> | null = null;
	try {
		body = (await res.json()) as ApiEnvelope<T>;
	} catch {
		body = null;
	}

	if (!res.ok || !body?.success) {
		throw new Error(body?.message ?? `Request failed (${res.status})`);
	}
	return body.data as T;
}
