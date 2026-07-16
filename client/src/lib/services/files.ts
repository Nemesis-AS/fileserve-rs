import type { FilerFile, FileSection } from '$lib/types';
import { MOCK_FILES } from '$lib/mock/data';
import { extOf, fileCategory, fileColor } from '$lib/utils/file';

// In-memory store for mock data (replaced by real API calls when backend is ready)
let _store: FilerFile[] = MOCK_FILES.map((f) => ({ ...f }));

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
	// TODO: Replace with real API calls. For now, throw to trigger mock fallback.
	const res = await fetch(`/api${path}`, init);
	if (!res.ok) throw new Error(`API ${res.status}`);
	return res.json() as Promise<T>;
}

export async function getFiles(): Promise<FilerFile[]> {
	try {
		return await apiFetch<FilerFile[]>('/files');
	} catch {
		return _store.map((f) => ({ ...f }));
	}
}

export async function uploadFile(
	_file: File,
	_makePublic: boolean,
	onProgress?: (pct: number) => void
): Promise<FilerFile> {
	// TODO: Real multipart upload to /api/files with XHR for progress
	return new Promise((resolve) => {
		let pct = 0;
		const tick = setInterval(() => {
			pct = Math.min(100, pct + 10 + Math.random() * 10);
			onProgress?.(pct);
			if (pct >= 100) {
				clearInterval(tick);
				const newFile: FilerFile = {
					id: 'f' + Date.now(),
					name: _file.name,
					ext: extOf(_file.name),
					category: fileCategory(_file.name),
					color: fileColor(_file.name),
					thumb: null,
					size: _file.size,
					modified: new Date().toISOString(),
					owner: 'me',
					public: _makePublic,
					trashed: false,
					trashedAt: null
				};
				_store = [newFile, ..._store];
				resolve(newFile);
			}
		}, 200);
	});
}

export async function renameFile(id: string, newName: string): Promise<FilerFile> {
	try {
		return await apiFetch<FilerFile>(`/files/${id}`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name: newName })
		});
	} catch {
		const idx = _store.findIndex((f) => f.id === id);
		if (idx < 0) throw new Error('File not found');
		_store[idx] = {
			..._store[idx],
			name: newName,
			ext: extOf(newName),
			category: fileCategory(newName),
			color: fileColor(newName)
		};
		return { ..._store[idx] };
	}
}

export async function trashFile(id: string): Promise<FilerFile> {
	try {
		return await apiFetch<FilerFile>(`/files/${id}/trash`, { method: 'POST' });
	} catch {
		const idx = _store.findIndex((f) => f.id === id);
		if (idx < 0) throw new Error('File not found');
		_store[idx] = { ..._store[idx], trashed: true, trashedAt: new Date().toISOString() };
		return { ..._store[idx] };
	}
}

export async function restoreFile(id: string): Promise<FilerFile> {
	try {
		return await apiFetch<FilerFile>(`/files/${id}/restore`, { method: 'POST' });
	} catch {
		const idx = _store.findIndex((f) => f.id === id);
		if (idx < 0) throw new Error('File not found');
		_store[idx] = { ..._store[idx], trashed: false, trashedAt: null };
		return { ..._store[idx] };
	}
}

export async function deleteFile(id: string): Promise<void> {
	try {
		await apiFetch(`/files/${id}`, { method: 'DELETE' });
	} catch {
		_store = _store.filter((f) => f.id !== id);
	}
}

export async function toggleShare(id: string, makePublic: boolean): Promise<FilerFile> {
	try {
		return await apiFetch<FilerFile>(`/files/${id}`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ public: makePublic })
		});
	} catch {
		const idx = _store.findIndex((f) => f.id === id);
		if (idx < 0) throw new Error('File not found');
		_store[idx] = { ..._store[idx], public: makePublic };
		return { ..._store[idx] };
	}
}

export function filterBySection(files: FilerFile[], section: FileSection): FilerFile[] {
	if (section === 'trash') return files.filter((f) => f.trashed);
	if (section === 'public') return files.filter((f) => !f.trashed && f.public);
	return files.filter((f) => !f.trashed);
}
