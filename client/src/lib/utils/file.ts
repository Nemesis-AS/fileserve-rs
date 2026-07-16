import type { FileCategory } from '$lib/types';

const EXT_COLOR: Record<FileCategory, string> = {
	img: '#0ea5e9',
	vid: '#a855f7',
	aud: '#ec4899',
	doc: '#2563eb',
	pdf: '#dc2626',
	zip: '#ca8a04',
	code: '#475569',
	txt: '#64748b',
	data: '#16a34a',
	other: '#6b7280'
};

const EXT_CAT: Record<string, FileCategory> = {
	png: 'img', jpg: 'img', jpeg: 'img', gif: 'img', webp: 'img', svg: 'img', heic: 'img',
	mp4: 'vid', mov: 'vid', mkv: 'vid', webm: 'vid',
	mp3: 'aud', flac: 'aud', wav: 'aud', m4a: 'aud',
	doc: 'doc', docx: 'doc', odt: 'doc', md: 'doc', rtf: 'doc',
	pdf: 'pdf',
	zip: 'zip', tar: 'zip', gz: 'zip', '7z': 'zip', rar: 'zip',
	py: 'code', js: 'code', ts: 'code', tsx: 'code', jsx: 'code', go: 'code', rs: 'code',
	c: 'code', cpp: 'code', h: 'code', sh: 'code', java: 'code', html: 'code', css: 'code',
	txt: 'txt', log: 'txt', conf: 'txt', cfg: 'txt', yaml: 'txt', yml: 'txt',
	csv: 'data', json: 'data', xlsx: 'data', tsv: 'data', sql: 'data'
};

export function extOf(name: string): string {
	const i = name.lastIndexOf('.');
	return i < 0 ? '' : name.slice(i + 1).toLowerCase();
}

export function fileCategory(name: string): FileCategory {
	return EXT_CAT[extOf(name)] ?? 'other';
}

export function fileColor(name: string): string {
	return EXT_COLOR[fileCategory(name)];
}

export function fmtSize(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	const units = ['KB', 'MB', 'GB', 'TB'];
	let v = bytes / 1024;
	let i = 0;
	while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
	return `${v.toFixed(v >= 10 || i === 0 ? 0 : 1)} ${units[i]}`;
}

export function fmtDate(d: string): string {
	const now = new Date();
	const date = new Date(d);
	const diff = (now.getTime() - date.getTime()) / 1000;
	if (diff < 60) return 'just now';
	if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
	if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
	if (diff < 86400 * 7) return `${Math.floor(diff / 86400)}d ago`;
	const opts: Intl.DateTimeFormatOptions = { month: 'short', day: 'numeric' };
	if (date.getFullYear() !== now.getFullYear()) opts.year = 'numeric';
	return date.toLocaleDateString('en-US', opts);
}

export function fmtDateLong(d: string): string {
	return new Date(d).toLocaleString('en-US', {
		year: 'numeric', month: 'short', day: 'numeric',
		hour: '2-digit', minute: '2-digit'
	});
}

export function initials(name: string): string {
	return name.split(' ').map((s) => s[0]).slice(0, 2).join('').toUpperCase();
}
