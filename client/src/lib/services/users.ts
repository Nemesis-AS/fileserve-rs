import type { User } from '$lib/types';
import { MOCK_USERS } from '$lib/mock/data';

let _store: User[] = MOCK_USERS.map((u) => ({ ...u }));

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
	const res = await fetch(`/api${path}`, init);
	if (!res.ok) throw new Error(`API ${res.status}`);
	return res.json() as Promise<T>;
}

export async function getUsers(): Promise<User[]> {
	try {
		return await apiFetch<User[]>('/users');
	} catch {
		return _store.map((u) => ({ ...u }));
	}
}

export async function createUser(data: Omit<User, 'id' | 'usedGB' | 'files' | 'lastSeen'>): Promise<User> {
	try {
		return await apiFetch<User>('/users', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(data)
		});
	} catch {
		const newUser: User = {
			...data,
			id: 'u' + Date.now(),
			usedGB: 0,
			files: 0,
			lastSeen: new Date().toISOString()
		};
		_store = [..._store, newUser];
		return { ...newUser };
	}
}

export async function updateUser(id: string, updates: Partial<User>): Promise<User> {
	try {
		return await apiFetch<User>(`/users/${id}`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(updates)
		});
	} catch {
		const idx = _store.findIndex((u) => u.id === id);
		if (idx < 0) throw new Error('User not found');
		_store[idx] = { ..._store[idx], ...updates };
		return { ..._store[idx] };
	}
}

export async function deleteUser(id: string): Promise<void> {
	try {
		await apiFetch(`/users/${id}`, { method: 'DELETE' });
	} catch {
		_store = _store.filter((u) => u.id !== id);
	}
}
