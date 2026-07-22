<script lang="ts">
	import { SearchInput } from './ui/search-input/index.js';
	import FileIcon from './FileIcon.svelte';
	import Icon from './Icon.svelte';
	import { clickOutside } from '$lib/actions/clickOutside';
	import { searchFiles } from '$lib/services/files';
	import { fmtSize } from '$lib/utils/file';
	import type { FilerFile } from '$lib/types';

	/**
	 * Topbar search with an autocomplete dropdown. Owns its own query + lookup so
	 * the host page only supplies navigation: `onPick` opens a suggested file,
	 * `onSubmit` opens the full results view. Presses `/` to focus via `ref`.
	 */
	let {
		initialValue = '',
		onPick,
		onSubmit,
		ref = $bindable(),
		class: klass
	}: {
		initialValue?: string;
		onPick: (file: FilerFile) => void;
		onSubmit: (query: string) => void;
		ref?: HTMLInputElement | null;
		class?: string;
	} = $props();

	const MAX_RESULTS = 8;
	const DEBOUNCE_MS = 180;

	let query = $state(initialValue);
	let results = $state<FilerFile[]>([]);
	let open = $state(false);
	let highlight = $state(0);

	$effect(() => {
		query = initialValue;
	});

	let token = 0;
	$effect(() => {
		const q = query.trim();
		if (!q) {
			results = [];
			return;
		}
		const mine = ++token;
		const timer = setTimeout(async () => {
			try {
				const found = await searchFiles(q);
				if (mine === token) results = found.slice(0, MAX_RESULTS);
			} catch {
				if (mine === token) results = [];
			}
		}, DEBOUNCE_MS);
		return () => clearTimeout(timer);
	});

	const searchRow = $derived(results.length);
	$effect(() => {
		results.length;
		highlight = searchRow;
	});

	function choose(idx: number) {
		const q = query.trim();
		if (!q) return;
		open = false;
		if (idx < results.length) onPick(results[idx]);
		else onSubmit(q);
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (open) {
				open = false;
				e.stopPropagation();
			}
			return;
		}
		if (e.key === 'Enter') {
			e.preventDefault();
			choose(open ? highlight : searchRow);
			return;
		}
		if (!open) return;
		const total = results.length + 1;
		if (e.key === 'ArrowDown') {
			highlight = (highlight + 1) % total;
			e.preventDefault();
		} else if (e.key === 'ArrowUp') {
			highlight = (highlight - 1 + total) % total;
			e.preventDefault();
		}
	}
</script>

<div class={`relative ${klass ?? ''}`} use:clickOutside={{ enabled: open, onOutside: () => (open = false) }}>
	<SearchInput
		bind:ref
		bind:value={query}
		placeholder="Search files…"
		kbd="/"
		oninput={() => (open = true)}
		onfocus={() => query.trim() && (open = true)}
		{onkeydown}
	/>

	{#if open && query.trim()}
		<div
			class="absolute top-[calc(100%+6px)] left-0 z-30 w-full min-w-[280px] overflow-hidden rounded-[9px] border border-edge bg-surface py-1 shadow-lg"
		>
			{#each results as f, i (f.id)}
				<button
					type="button"
					class="flex w-full items-center gap-2.5 px-2.5 py-1.5 text-left text-[13px] text-ink data-[active=1]:bg-row-hover"
					data-active={highlight === i ? '1' : '0'}
					onmousemove={() => (highlight = i)}
					onclick={() => choose(i)}
				>
					<FileIcon file={f} />
					<span class="min-w-0 flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{f.name}</span>
					<span class="shrink-0 font-code text-[11px] text-ink-faint">{fmtSize(f.size)}</span>
				</button>
			{/each}

			{#if results.length === 0}
				<div class="px-2.5 py-1.5 text-[12.5px] text-ink-faint">No matching files</div>
			{/if}

			<div class="my-1 border-t border-edge"></div>

			<button
				type="button"
				class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-ink data-[active=1]:bg-row-hover"
				data-active={highlight === searchRow ? '1' : '0'}
				onmousemove={() => (highlight = searchRow)}
				onclick={() => choose(searchRow)}
			>
				<Icon name="Search" size={14} class="shrink-0 text-ink-muted" />
				<span class="min-w-0 flex-1 truncate">Search for &ldquo;{query.trim()}&rdquo;</span>
				<Icon name="ChevronR" size={13} class="shrink-0 text-ink-faint" />
			</button>
		</div>
	{/if}
</div>
