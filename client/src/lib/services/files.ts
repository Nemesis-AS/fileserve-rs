import * as tus from 'tus-js-client';
import type { FilerFile, FileSection } from '$lib/types';
import { extOf, fileCategory, fileColor } from '$lib/utils/file';
import { handleUnauthorized } from './session';

const API = '/api/v1';

/** Envelope every backend endpoint wraps its payload in. */
interface ApiEnvelope<T> {
	success: boolean;
	message: string | null;
	data: T | null;
}

/** Row shape returned by the `files` endpoints. */
interface FileRecord {
	id: string;
	file_name: string;
	mime_type: string;
	file_size: number;
	checksum: string;
	owner_uname: string;
	public: boolean;
	created_at: string | null;
	deleted_at: string | null;
}

async function unwrap<T>(res: Response, fallbackMsg: string): Promise<T> {
	if (res.status === 401) handleUnauthorized();

	let body: ApiEnvelope<T> | null = null;
	try {
		body = (await res.json()) as ApiEnvelope<T>;
	} catch {
		// Non-JSON response (proxy error, server down) — fall through.
	}

	if (!res.ok || !body?.success) {
		throw new Error(body?.message ?? fallbackMsg);
	}
	if (body.data === null || body.data === undefined) {
		throw new Error(fallbackMsg);
	}
	return body.data;
}

/**
 * The backend stores only what it needs to serve bytes; everything the UI shows
 * beyond that (icon category, colour, extension) is derived from the name here.
 *
 * `public` reflects the persisted per-file flag: a public file is listed in the
 * Public section and downloadable by any signed-in user. Separate from the
 * time-limited JWT share links (`POST /files/{id}/share`).
 */
function toFilerFile(r: FileRecord): FilerFile {
	return {
		id: r.id,
		name: r.file_name,
		ext: extOf(r.file_name),
		category: fileCategory(r.file_name),
		color: fileColor(r.file_name),
		thumb: null,
		size: r.file_size,
		modified: r.created_at ?? new Date().toISOString(),
		owner: r.owner_uname,
		public: r.public,
		trashed: r.deleted_at !== null,
		trashedAt: r.deleted_at
	};
}

export async function getFiles(): Promise<FilerFile[]> {
	const res = await fetch(`${API}/files/my`, { credentials: 'include' });
	const records = await unwrap<FileRecord[]>(res, 'Could not load files');
	return records.map(toFilerFile);
}

/**
 * Public files across every owner — the shared Public section. Unlike
 * {@link getFiles}, these aren't scoped to the current user, but the endpoint
 * still requires a session.
 */
export async function getPublicFiles(): Promise<FilerFile[]> {
	const res = await fetch(`${API}/files/public`, { credentials: 'include' });
	const records = await unwrap<FileRecord[]>(res, 'Could not load public files');
	return records.map(toFilerFile);
}

/**
 * Server-side search over the user's own, non-trashed files (matches name or
 * directory, capped at 50 rows by the backend). Powers the topbar autocomplete
 * and the `?q=` results view, replacing the old in-memory name filter.
 */
export async function searchFiles(query: string): Promise<FilerFile[]> {
	const res = await fetch(`${API}/files/search?filename=${encodeURIComponent(query)}`, {
		credentials: 'include'
	});
	const records = await unwrap<FileRecord[]>(res, 'Search failed');
	return records.map(toFilerFile);
}

const TUS_CHUNK_SIZE = 5 * 1024 * 1024;

export function uploadFile(
	file: File,
	_makePublic: boolean,
	onProgress?: (pct: number) => void,
	signal?: AbortSignal
): Promise<void> {
	return new Promise((resolve, reject) => {
		const upload = new tus.Upload(file, {
			endpoint: `${API}/files/upload`,
			chunkSize: TUS_CHUNK_SIZE,
			retryDelays: [0, 1000, 3000, 5000],
			onBeforeRequest: (req) => {
				const xhr = req.getUnderlyingObject();
				if (xhr instanceof XMLHttpRequest) xhr.withCredentials = true;
			},
			metadata: {
				file_name: file.name,
				mime_type: file.type || 'application/octet-stream',
				file_dir: '/'
			},
			onError: (err) => reject(err instanceof Error ? err : new Error(String(err))),
			onProgress: (sent, total) => onProgress?.(total ? (sent / total) * 100 : 0),
			onSuccess: () => {
				onProgress?.(100);
				resolve();
			}
		});

		signal?.addEventListener('abort', () => upload.abort(), { once: true });

		upload.start();
	});
}

/**
 * Same-origin URL, so the session cookie rides along; the server sets
 * `Content-Disposition: attachment`, which is what actually triggers the save.
 */
export function downloadUrl(id: string): string {
	return `${API}/files/${id}/download`;
}

/**
 * Same endpoint as {@link downloadUrl}, but asks the server for
 * `Content-Disposition: inline` so the browser renders it in-page (image
 * `src`, PDF `iframe`, or a `fetch` for text) instead of triggering a save.
 */
export function previewUrl(id: string): string {
	return `${API}/files/${id}/download?inline=true`;
}

export function downloadFile(file: FilerFile): void {
	const a = document.createElement('a');
	a.href = downloadUrl(file.id);
	a.download = file.name;
	document.body.appendChild(a);
	a.click();
	a.remove();
}

export interface ShareToken {
	/** Opaque JWT to append to the download URL as `?token=`. */
	token: string;
	/** ISO-8601 instant at which the token stops working. */
	expiresAt: string;
}

/**
 * Mints a time-limited share token for a file the user owns. Unlike the `public`
 * flag (which requires the recipient to be signed in), the token embeds in the
 * download URL and works without a session until it expires. `expiresInMinutes`
 * is clamped server-side to 1 minute … 7 days. Tokens are stateless JWTs, so
 * there's no server-side list or revocation yet.
 */
export async function createShareLink(id: string, expiresInMinutes: number): Promise<ShareToken> {
	const res = await fetch(`${API}/files/${id}/share`, {
		method: 'POST',
		credentials: 'include',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ expires_in_minutes: expiresInMinutes })
	});
	const data = await unwrap<{ token: string; expires_at: string }>(
		res,
		'Could not create share link'
	);
	return { token: data.token, expiresAt: data.expires_at };
}

async function mutate(
	path: string,
	method: string,
	fallbackMsg: string,
	body?: unknown
): Promise<FilerFile> {
	const res = await fetch(`${API}/files/${path}`, {
		method,
		credentials: 'include',
		headers: body ? { 'Content-Type': 'application/json' } : undefined,
		body: body ? JSON.stringify(body) : undefined
	});
	return toFilerFile(await unwrap<FileRecord>(res, fallbackMsg));
}

export function renameFile(id: string, newName: string): Promise<FilerFile> {
	return mutate(id, 'PATCH', 'Could not rename file', { name: newName });
}

export function trashFile(id: string): Promise<FilerFile> {
	return mutate(`${id}/trash`, 'POST', 'Could not move file to trash');
}

export function restoreFile(id: string): Promise<FilerFile> {
	return mutate(`${id}/restore`, 'POST', 'Could not restore file');
}

export async function deleteFile(id: string): Promise<void> {
	const res = await fetch(`${API}/files/${id}`, { method: 'DELETE', credentials: 'include' });
	if (res.status === 401) handleUnauthorized();
	if (!res.ok) {
		const body = (await res.json().catch(() => null)) as ApiEnvelope<never> | null;
		throw new Error(body?.message ?? 'Could not delete file');
	}
}

/**
 * Persists the file's visibility. Owner-only server-side; a non-owner attempt
 * rejects with the endpoint's error, surfaced by the caller.
 */
export function toggleShare(file: FilerFile, makePublic: boolean): Promise<FilerFile> {
	return mutate(`${file.id}/public`, 'PATCH', 'Could not update sharing', { public: makePublic });
}

/**
 * Filters the *current user's* files for the `my`/`trash` sections. The `public`
 * section is a separate cross-owner listing (see {@link getPublicFiles}) and is
 * loaded directly rather than filtered from this set.
 */
export function filterBySection(files: FilerFile[], section: FileSection): FilerFile[] {
	if (section === 'trash') return files.filter((f) => f.trashed);
	if (section === 'public') return files.filter((f) => !f.trashed && f.public);
	return files.filter((f) => !f.trashed);
}
