<script lang="ts">
	import { fly } from 'svelte/transition';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { tv } from './ui/tv.js';

	/**
	 * Port of `.toast`. Kept faithfully, including the quirk that each toast is itself
	 * `fixed` — so the wrapper's flex/gap never applies and stacked toasts overlap rather
	 * than pile up. Not fixing that here: it's a behaviour change, not a restyle.
	 */
	const toast = tv({
		base: 'fixed bottom-4 left-1/2 z-200 flex -translate-x-1/2 items-center gap-3 rounded-lg bg-ink px-[14px] py-[9px] text-[13px] font-[450] whitespace-nowrap text-surface shadow-pop'
	});
</script>

<div
	class="pointer-events-none fixed bottom-4 left-1/2 z-200 flex -translate-x-1/2 flex-col items-center gap-2"
>
	{#each toastStore.toasts as t (t.id)}
		<div class={toast({ class: 'pointer-events-auto' })} transition:fly={{ y: 12, duration: 220 }}>
			<span>{t.msg}</span>
			{#if t.onUndo}
				<button
					class="cursor-pointer border-0 bg-transparent p-0 font-medium text-accent dark:text-accent-ink"
					onclick={() => toastStore.undo(t.id)}
				>
					Undo
				</button>
			{/if}
		</div>
	{/each}
</div>
