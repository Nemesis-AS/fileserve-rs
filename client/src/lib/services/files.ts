import type { FilerFile, FileSection } from '$lib/types';
import { extOf, fileCategory, fileColor } from '$lib/utils/file';

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
	created_at: string | null;
	deleted_at: string | null;
}

async function unwrap<T>(res: Response, fallbackMsg: string): Promise<T> {
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
 * `public` is hardcoded false: sharing is token-based on the server
 * (`POST /files/{id}/share` mints a time-limited link) and there is no per-file
 * public flag in the schema for it to reflect.
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
		public: false,
		trashed: r.deleted_at !== null,
		trashedAt: r.deleted_at
	};
}

export async function getFiles(): Promise<FilerFile[]> {
	const res = await fetch(`${API}/files/my`, { credentials: 'include' });
	const records = await unwrap<FileRecord[]>(res, 'Could not load files');
	return records.map(toFilerFile);
}

// ── Uploads (TUS 1.0.0) ────────────────────────────────────────────────────
// The server implements the TUS creation extension: POST to allocate, then PATCH
// successive chunks at an explicit offset. We chunk rather than sending one body
// because `fetch` reports no upload progress — each completed chunk is a tick.

const TUS_CHUNK_SIZE = 5 * 1024 * 1024;

/** TUS metadata values are base64, and `btoa` chokes on non-Latin-1 names. */
function encodeMetadata(fields: Record<string, string>): string {
	return Object.entries(fields)
		.map(([key, value]) => {
			const bytes = new TextEncoder().encode(value);
			const binary = Array.from(bytes, (b) => String.fromCharCode(b)).join('');
			return `${key} ${btoa(binary)}`;
		})
		.join(',');
}

export async function uploadFile(
	file: File,
	_makePublic: boolean,
	onProgress?: (pct: number) => void,
	signal?: AbortSignal
): Promise<void> {
	const createRes = await fetch(`${API}/files/upload`, {
		method: 'POST',
		credentials: 'include',
		signal,
		headers: {
			'Tus-Resumable': '1.0.0',
			'Upload-Length': String(file.size),
			'Upload-Metadata': encodeMetadata({
				file_name: file.name,
				mime_type: file.type || 'application/octet-stream',
				file_dir: '/'
			})
		}
	});

	if (createRes.status !== 201) {
		throw new Error(await uploadError(createRes, 'Could not start upload'));
	}

	// Server returns the upload URL as a root-relative path.
	const location = createRes.headers.get('Location');
	if (!location) throw new Error('Upload did not return a location');

	let offset = 0;
	// A zero-byte file sends no chunk. The server only writes its `files` row when
	// a chunk carries the upload to its declared length, so empty files currently
	// never materialize — an upstream gap, not something the client can paper over.
	while (offset < file.size) {
		const chunk = file.slice(offset, Math.min(offset + TUS_CHUNK_SIZE, file.size));

		const patchRes = await fetch(location, {
			method: 'PATCH',
			credentials: 'include',
			signal,
			headers: {
				'Tus-Resumable': '1.0.0',
				'Content-Type': 'application/offset+octet-stream',
				'Upload-Offset': String(offset)
			},
			body: chunk
		});

		if (patchRes.status !== 204) {
			throw new Error(await uploadError(patchRes, 'Upload failed'));
		}

		// Trust the server's offset over our own arithmetic — it is authoritative
		// about how much of the chunk actually landed.
		const next = Number(patchRes.headers.get('Upload-Offset'));
		offset = Number.isFinite(next) && next > offset ? next : offset + chunk.size;
		onProgress?.(Math.min(100, (offset / file.size) * 100));
	}

	onProgress?.(100);
}

/** TUS errors use the same envelope as the JSON endpoints, when they have a body at all. */
async function uploadError(res: Response, fallback: string): Promise<string> {
	try {
		const body = (await res.json()) as ApiEnvelope<never>;
		return body?.message ?? fallback;
	} catch {
		return `${fallback} (${res.status})`;
	}
}

// ── Downloads ──────────────────────────────────────────────────────────────

/**
 * Same-origin URL, so the session cookie rides along; the server sets
 * `Content-Disposition: attachment`, which is what actually triggers the save.
 */
export function downloadUrl(id: string): string {
	return `${API}/files/${id}/download`;
}

export function downloadFile(file: FilerFile): void {
	const a = document.createElement('a');
	a.href = downloadUrl(file.id);
	a.download = file.name;
	document.body.appendChild(a);
	a.click();
	a.remove();
}

// ── Mutations ──────────────────────────────────────────────────────────────

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
	if (!res.ok) {
		const body = (await res.json().catch(() => null)) as ApiEnvelope<never> | null;
		throw new Error(body?.message ?? 'Could not delete file');
	}
}

/**
 * Not yet persisted: the schema has no public flag, so this only updates the
 * local copy. Wiring it up means either a `public` column or reworking the UI
 * around the token-based share links the server already mints.
 */
export async function toggleShare(file: FilerFile, makePublic: boolean): Promise<FilerFile> {
	return { ...file, public: makePublic };
}

export function filterBySection(files: FilerFile[], section: FileSection): FilerFile[] {
	if (section === 'trash') return files.filter((f) => f.trashed);
	if (section === 'public') return files.filter((f) => !f.trashed && f.public);
	return files.filter((f) => !f.trashed);
}
