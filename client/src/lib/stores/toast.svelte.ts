import type { Toast } from '$lib/types';

let _toasts = $state<Toast[]>([]);
const timers = new Map<string, ReturnType<typeof setTimeout>>();

export const toastStore = {
	get toasts() { return _toasts; },

	show(msg: string, onUndo?: () => void, duration = 5000): string {
		const id = crypto.randomUUID();
		_toasts = [..._toasts, { id, msg, onUndo }];
		const t = setTimeout(() => toastStore.dismiss(id), duration);
		timers.set(id, t);
		return id;
	},

	dismiss(id: string) {
		clearTimeout(timers.get(id));
		timers.delete(id);
		_toasts = _toasts.filter((t) => t.id !== id);
	},

	undo(id: string) {
		const toast = _toasts.find((t) => t.id === id);
		if (toast?.onUndo) toast.onUndo();
		toastStore.dismiss(id);
	}
};
