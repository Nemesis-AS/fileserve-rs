import type { User } from '$lib/types';
import { MOCK_USERS } from '$lib/mock/data';

export interface LoginResult {
	user: User;
	token: string;
}

export async function login(username: string, _password: string): Promise<LoginResult> {
	try {
		const res = await fetch('/api/auth/login', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ username, password: _password })
		});
		if (!res.ok) throw new Error('Invalid credentials');
		return res.json() as Promise<LoginResult>;
	} catch {
		// Mock: find user by username (any password accepted in dev)
		const user = MOCK_USERS.find((u) => u.username === username);
		if (!user) throw new Error('User not found');
		if (user.status === 'suspended') throw new Error('Account suspended');
		return { user: { ...user }, token: 'mock-token-' + user.id };
	}
}

export async function logout(): Promise<void> {
	try {
		await fetch('/api/auth/logout', { method: 'POST' });
	} catch {
		// Mock: no-op
	}
}
