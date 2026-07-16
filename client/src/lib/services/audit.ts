import type { AuditEvent } from '$lib/types';
import { MOCK_AUDIT } from '$lib/mock/data';

async function apiFetch<T>(path: string): Promise<T> {
	const res = await fetch(`/api${path}`);
	if (!res.ok) throw new Error(`API ${res.status}`);
	return res.json() as Promise<T>;
}

export async function getAuditEvents(): Promise<AuditEvent[]> {
	try {
		return await apiFetch<AuditEvent[]>('/audit');
	} catch {
		return MOCK_AUDIT.map((e) => ({ ...e }));
	}
}
