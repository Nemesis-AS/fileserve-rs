/** Shared helpers for the custom audio/video players. */

/** `s` seconds → `m:ss`, or `h:mm:ss` once past an hour. */
export function formatTime(s: number): string {
	if (!isFinite(s) || s < 0) return '0:00';
	const total = Math.floor(s);
	const h = Math.floor(total / 3600);
	const m = Math.floor((total % 3600) / 60);
	const sec = total % 60;
	const mm = h ? m.toString().padStart(2, '0') : m.toString();
	const rest = `${mm}:${sec.toString().padStart(2, '0')}`;
	return h ? `${h}:${rest}` : rest;
}

/**
 * Pointer-drag behaviour shared by the seek and volume tracks. `apply` receives
 * a 0–1 ratio of where along the track the pointer is. Pointer capture keeps the
 * drag responsive even when the cursor leaves the (thin) track.
 */
export function makeSlider(apply: (ratio: number) => void) {
	let dragging = false;
	const ratioAt = (e: PointerEvent) => {
		const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
		return Math.min(1, Math.max(0, (e.clientX - rect.left) / rect.width));
	};
	return {
		down(e: PointerEvent) {
			dragging = true;
			(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
			apply(ratioAt(e));
		},
		move(e: PointerEvent) {
			if (dragging) apply(ratioAt(e));
		},
		up(e: PointerEvent) {
			dragging = false;
			(e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
		}
	};
}
