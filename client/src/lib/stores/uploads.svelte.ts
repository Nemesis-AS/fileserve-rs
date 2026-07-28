import { uploadFile } from '$lib/services/files';
import { extOf, fileColor } from '$lib/utils/file';
import { countsStore } from './counts.svelte';
import { quotaStore } from './quota.svelte';

export interface UploadItem {
	id: string;
	name: string;
	size: number;
	ext: string;
	color: string;
	progress: number;
	state: 'uploading' | 'done' | 'error';
	error?: string;
}

/**
 * Transfers live here rather than inside the upload modal so they survive the
 * dialog closing and navigation between sections — the dock (rendered by the
 * app layout) is just a view over this queue.
 */
let _items = $state<UploadItem[]>([]);
let _open = $state(false);
let _collapsed = $state(false);
let _finished = $state(0);

// Never read from a reactive context — it's just bookkeeping so cancel/dismiss can
// reach the in-flight tus upload — so a plain Map is right over SvelteMap.
// eslint-disable-next-line svelte/prefer-svelte-reactivity
const aborters = new Map<string, AbortController>();
let seq = 0;

function patch(id: string, changes: Partial<UploadItem>) {
	_items = _items.map((it) => (it.id === id ? { ...it, ...changes } : it));
}

async function run(id: string, file: File) {
	const aborter = new AbortController();
	aborters.set(id, aborter);
	try {
		await uploadFile(file, (pct) => patch(id, { progress: pct }), aborter.signal);
		patch(id, { progress: 100, state: 'done' });
		_finished++;
		void quotaStore.refresh();
		void countsStore.refresh();
	} catch (e) {
		if (aborter.signal.aborted) return;
		patch(id, { state: 'error', error: e instanceof Error ? e.message : 'Upload failed' });
	} finally {
		aborters.delete(id);
	}
}

export const uploadsStore = {
	get items() {
		return _items;
	},
	get open() {
		return _open;
	},
	get collapsed() {
		return _collapsed;
	},
	/** Bumped once per successful upload; views watch it to reload their listing. */
	get finished() {
		return _finished;
	},
	get activeCount() {
		return _items.filter((it) => it.state === 'uploading').length;
	},
	get doneCount() {
		return _items.filter((it) => it.state === 'done').length;
	},
	get errorCount() {
		return _items.filter((it) => it.state === 'error').length;
	},

	/** Queues files and starts them immediately, revealing the dock expanded. */
	add(files: FileList | File[]) {
		const list = Array.from(files);
		if (!list.length) return;

		_open = true;
		_collapsed = false;

		for (const file of list) {
			const item: UploadItem = {
				id: `u${++seq}`,
				name: file.name,
				size: file.size,
				ext: extOf(file.name),
				color: fileColor(file.name),
				progress: 0,
				state: 'uploading'
			};
			_items = [..._items, item];
			void run(item.id, file);
		}
	},

	/** Aborts the transfer if it's still running, then drops the row. */
	remove(id: string) {
		aborters.get(id)?.abort();
		aborters.delete(id);
		_items = _items.filter((it) => it.id !== id);
		if (!_items.length) _open = false;
	},

	toggleCollapsed() {
		_collapsed = !_collapsed;
	},

	/** Dismisses the dock, cancelling anything still in flight. */
	dismiss() {
		aborters.forEach((a) => a.abort());
		aborters.clear();
		_items = [];
		_open = false;
		_collapsed = false;
	}
};
