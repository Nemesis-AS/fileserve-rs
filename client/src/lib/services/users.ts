import type { User } from '$lib/types';
import { handleUnauthorized } from './session';

const API = '/api/v1';

/** Envelope every backend endpoint wraps its payload in. */
interface ApiEnvelope<T> {
	success: boolean;
	message: string | null;
	data: T | null;
}

/**
 * Calls a `/api/v1` endpoint and unwraps the `{ success, message, data }`
 * envelope, surfacing the server's `message` as the thrown error. Unlike the
 * file services there is no mock fallback: these are admin-only mutations, so a
 * failure (not signed in, not an admin, server down) must be visible rather
 * than silently masked by seed data.
 */
async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
	const res = await fetch(`${API}${path}`, { credentials: 'include', ...init });

	if (res.status === 401) handleUnauthorized();

	let body: ApiEnvelope<T> | null = null;
	try {
		body = (await res.json()) as ApiEnvelope<T>;
	} catch {
		// Non-JSON response (proxy error, server down) — fall through.
	}

	if (!res.ok || !body?.success) {
		throw new Error(body?.message ?? `Request failed (${res.status})`);
	}
	return body.data as T;
}

export async function getUsers(): Promise<User[]> {
	return apiFetch<User[]>('/users');
}

export interface Quota {
	/** GB; null means no limit. */
	quotaGB: number | null;
	usedGB: number;
	files: number;
}

/**
 * The current user's live quota + usage. Fetched fresh (not read from the
 * session) so an admin's mid-session quota change is reflected on the next call.
 */
export async function getQuota(): Promise<Quota> {
	return apiFetch<Quota>('/auth/quota');
}

/**
 * The authenticated user, resolved from the auth cookie. Used to rehydrate the
 * session on app load so role/quota reflect the server rather than (possibly
 * stale or tampered) client storage. Throws if the cookie is missing/invalid.
 */
export async function getMe(): Promise<User> {
	return apiFetch<User>('/auth/me');
}

/**
 * Update the caller's own profile (currently just the display name) via the
 * self-scoped route, and return the refreshed user. Unlike `updateUser`, this
 * needs no admin rights — it only ever touches the caller's own account.
 */
export async function updateProfile(updates: { name: string }): Promise<User> {
	return apiFetch<User>('/auth/me', {
		method: 'PATCH',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(updates)
	});
}

/** Change the caller's own password. Throws with the server's reason on failure. */
export async function changePassword(currentPassword: string, newPassword: string): Promise<void> {
	await apiFetch<null>('/auth/password', {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ current_password: currentPassword, new_password: newPassword })
	});
}

/** The subset the create form collects; `password` is optional (server mints one when blank). */
type CreateUserInput = Omit<User, 'id' | 'usedGB' | 'files' | 'lastSeen'> & { password?: string };

/**
 * The created user, plus `tempPassword` when the server generated one. That
 * password is stored only as a hash, so this response is the sole opportunity
 * to show it — the caller must surface it before navigating away. Absent when
 * the admin supplied their own password.
 */
export interface CreatedUser extends User {
	tempPassword?: string;
}

export async function createUser(data: CreateUserInput): Promise<CreatedUser> {
	return apiFetch<CreatedUser>('/users', {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(data)
	});
}

export async function updateUser(
	id: string,
	updates: Partial<User> & { password?: string }
): Promise<User> {
	return apiFetch<User>(`/users/${id}`, {
		method: 'PATCH',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(updates)
	});
}

export async function deleteUser(id: string): Promise<void> {
	await apiFetch<null>(`/users/${id}`, { method: 'DELETE' });
}
