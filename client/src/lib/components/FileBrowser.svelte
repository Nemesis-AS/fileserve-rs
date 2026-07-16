<script lang="ts">
	import { flip } from 'svelte/animate';
	import Icon from './Icon.svelte';
	import FileIcon from './FileIcon.svelte';
	import GridTile from './GridTile.svelte';
	import EmptyState from './EmptyState.svelte';
	import type { FilerFile, FileSection, ViewMode, SortKey, SortDir } from '$lib/types';
	import { fmtSize, fmtDate } from '$lib/utils/file';
	import { table, cell } from './ui/table/index.js';
	import { IconButton } from './ui/icon-button/index.js';
	import { Segmented } from './ui/segmented/index.js';

	let {
		files,
		view,
		section,
		search = '',
		activeId = null,
		onActiveId,
		onViewChange,
		onView,
		onDownload,
		onRename,
		onDelete,
		onProperties,
		onUpload
	}: {
		files: FilerFile[];
		view: ViewMode;
		section: FileSection;
		search?: string;
		activeId?: string | null;
		onActiveId: (id: string | null) => void;
		onViewChange?: (v: ViewMode) => void;
		onView: (f: FilerFile) => void;
		onDownload: (f: FilerFile) => void;
		onRename: (f: FilerFile) => void;
		onDelete: (f: FilerFile) => void;
		onProperties: (f: FilerFile) => void;
		onUpload: () => void;
	} = $props();

	let sortKey = $state<SortKey>('modified');
	let sortDir = $state<SortDir>('desc');
	let pageSize = $state(40);
	let scrollEl: HTMLDivElement | undefined = $state();
	const rowEls: Record<string, HTMLTableRowElement> = {};

	const sorted = $derived.by(() => {
		const arr = files.slice();
		arr.sort((a, b) => {
			let av: string | number, bv: string | number;
			if (sortKey === 'name') {
				av = a.name.toLowerCase();
				bv = b.name.toLowerCase();
			} else if (sortKey === 'size') {
				av = a.size;
				bv = b.size;
			} else if (sortKey === 'type') {
				av = a.ext;
				bv = b.ext;
			} else {
				av = a.modified;
				bv = b.modified;
			}
			if (av < bv) return sortDir === 'asc' ? -1 : 1;
			if (av > bv) return sortDir === 'asc' ? 1 : -1;
			return 0;
		});
		return arr;
	});

	const visible = $derived(sorted.slice(0, pageSize));

	$effect(() => {
		// Reset paging when list changes
		files.length;
		sortKey;
		sortDir;
		view;
		pageSize = 40;
	});

	$effect(() => {
		if (!activeId) return;
		const el = rowEls[activeId];
		if (el && scrollEl) {
			const r1 = el.getBoundingClientRect();
			const r2 = scrollEl.getBoundingClientRect();
			if (r1.top < r2.top + 60) scrollEl.scrollBy({ top: r1.top - r2.top - 60 });
			if (r1.bottom > r2.bottom) scrollEl.scrollBy({ top: r1.bottom - r2.bottom + 24 });
		}
	});

	function onScroll(e: Event) {
		const el = e.currentTarget as HTMLDivElement;
		if (el.scrollTop + el.clientHeight > el.scrollHeight - 200 && pageSize < sorted.length) {
			pageSize = Math.min(pageSize + 40, sorted.length);
		}
	}

	function setSort(k: SortKey) {
		if (k === sortKey) sortDir = sortDir === 'asc' ? 'desc' : 'asc';
		else {
			sortKey = k;
			sortDir = k === 'name' ? 'asc' : 'desc';
		}
	}

	const sortLabel = $derived(
		{
			name: 'Name',
			type: 'Type',
			size: 'Size',
			modified: 'Modified'
		}[sortKey]
	);

	const t = table({ variant: 'files' });

	const VIEW_OPTIONS: { value: ViewMode; label: string }[] = [
		{ value: 'list', label: 'List view' },
		{ value: 'grid', label: 'Grid view' }
	];
</script>

{#if files.length === 0}
	<EmptyState {section} {search} onUpload={section !== 'trash' ? onUpload : null} />
{:else}
	<!-- .filebar -->
	<div
		class="flex h-10 shrink-0 items-center gap-2 border-b border-edge px-page text-[12.5px] text-ink-muted"
	>
		<span class="font-medium text-ink">{files.length} {files.length === 1 ? 'file' : 'files'}</span>
		{#if search}
			<span>· matching "{search}"</span>
		{/if}
		<div class="flex-1"></div>

		<Segmented value={view} options={VIEW_OPTIONS} onchange={(v) => onViewChange?.(v)} class="mr-2">
			{#snippet item(v)}
				{#if v === 'list'}
					<svg
						viewBox="0 0 16 16"
						fill="none"
						stroke="currentColor"
						stroke-width="1.6"
						stroke-linecap="round"
					>
						<path d="M3 4h10M3 8h10M3 12h10" />
					</svg>
				{:else}
					<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6">
						<rect x="2.5" y="2.5" width="4.5" height="4.5" rx="1" />
						<rect x="9" y="2.5" width="4.5" height="4.5" rx="1" />
						<rect x="2.5" y="9" width="4.5" height="4.5" rx="1" />
						<rect x="9" y="9" width="4.5" height="4.5" rx="1" />
					</svg>
				{/if}
			{/snippet}
		</Segmented>

		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<span
			class="flex cursor-pointer items-center gap-1 rounded-[5px] px-1.5 py-0.5 transition-colors duration-100 select-none hover:bg-row-hover hover:text-ink"
			onclick={() => setSort(sortKey)}
		>
			{sortLabel}
			{#if sortDir === 'desc'}<Icon name="ArrowDown" size={13} />{:else}<Icon
					name="ArrowUp"
					size={13}
				/>{/if}
		</span>
	</div>

	<div class="scroll-area min-h-0 flex-1 overflow-auto" bind:this={scrollEl} onscroll={onScroll}>
		{#if view === 'list'}
			<table class={t.root()}>
				<thead>
					<tr>
						<th class={t.th()} onclick={() => setSort('name')}>
							Name {#if sortKey === 'name'}<span class="ml-0.5 text-[11px] text-accent"
									>{sortDir === 'asc' ? '↑' : '↓'}</span
								>{/if}
						</th>
						<th class={t.th({ class: 'w-[100px]' })} onclick={() => setSort('type')}>
							Type {#if sortKey === 'type'}<span class="ml-0.5 text-[11px] text-accent"
									>{sortDir === 'asc' ? '↑' : '↓'}</span
								>{/if}
						</th>
						<th class={t.th({ class: 'w-[110px]' })} onclick={() => setSort('size')}>
							Size {#if sortKey === 'size'}<span class="ml-0.5 text-[11px] text-accent"
									>{sortDir === 'asc' ? '↑' : '↓'}</span
								>{/if}
						</th>
						<th class={t.th({ class: 'w-[130px]' })} onclick={() => setSort('modified')}>
							Modified {#if sortKey === 'modified'}<span class="ml-0.5 text-[11px] text-accent"
									>{sortDir === 'asc' ? '↑' : '↓'}</span
								>{/if}
						</th>
						{#if section === 'public'}<th class={t.th({ class: 'w-[110px]' })}>Owner</th>{/if}
						<th class={t.th({ class: 'w-[130px]' })}></th>
					</tr>
				</thead>
				<tbody>
					{#each visible as f (f.id)}
						<!--
						  `group/row` powers the row-action reveal below — the old
						  `.table tbody tr:hover .row-actions` + `[data-active]` pair.
						-->
						<tr
							class={t.tr({ class: 'group/row' })}
							animate:flip={{ duration: 160 }}
							data-active={activeId === f.id ? '1' : '0'}
							onclick={() => onActiveId(f.id)}
							ondblclick={() => onView(f)}
							bind:this={rowEls[f.id]}
						>
							<td class={t.td()}>
								<div class="flex min-w-0 items-center gap-[9px]">
									<FileIcon file={f} />
									<span class="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap"
										>{f.name}</span
									>
									{#if f.public && section !== 'public'}
										<span
											class="ml-1.5 shrink-0 rounded-[3px] bg-accent-soft px-[5px] py-px text-[9.5px] font-semibold whitespace-nowrap text-accent-ink uppercase"
										>
											Shared
										</span>
									{/if}
								</div>
							</td>
							<td class={t.td({ class: cell({ variant: 'type' }) })}>{f.ext}</td>
							<td class={t.td({ class: cell({ variant: 'size' }) })}>{fmtSize(f.size)}</td>
							<td class={t.td({ class: cell({ variant: 'date' }) })}>
								{section === 'trash' && f.trashedAt
									? `del · ${fmtDate(f.trashedAt)}`
									: fmtDate(f.modified)}
							</td>
							{#if section === 'public'}
								<td class={t.td({ class: cell({ variant: 'owner' }) })}>
									{f.owner === 'me' ? 'You' : f.owner}
								</td>
							{/if}
							<td class={t.td({ class: cell({ variant: 'actions' }) })}>
								<!-- svelte-ignore a11y_click_events_have_key_events -->
								<!-- svelte-ignore a11y_no_static_element_interactions -->
								<div
									class="flex justify-end gap-0.5 opacity-0 transition-opacity duration-100 group-hover/row:opacity-100 group-data-[active=1]/row:opacity-100"
									onclick={(e) => e.stopPropagation()}
								>
									<IconButton variant="row" size="sm" title="Open" onclick={() => onView(f)}>
										<Icon name="Eye" size={14} />
									</IconButton>
									<IconButton
										variant="row"
										size="sm"
										title="Download"
										onclick={() => onDownload(f)}
									>
										<Icon name="Download" size={14} />
									</IconButton>
									{#if section !== 'trash'}
										<IconButton variant="row" size="sm" title="Rename" onclick={() => onRename(f)}>
											<Icon name="Pencil" size={14} />
										</IconButton>
									{/if}
									<IconButton
										variant="row"
										size="sm"
										title="Properties"
										onclick={() => onProperties(f)}
									>
										<Icon name="Info" size={14} />
									</IconButton>
									<IconButton variant="row" size="sm" title="Delete" onclick={() => onDelete(f)}>
										<Icon name="Trash" size={14} />
									</IconButton>
								</div>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{:else}
			<div
				class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-[14px] px-page pt-4 pb-page"
			>
				{#each visible as f (f.id)}
					<div animate:flip={{ duration: 160 }}>
						<GridTile
							file={f}
							active={activeId === f.id}
							{section}
							onclick={() => onActiveId(f.id)}
							ondblclick={() => onView(f)}
							{onView}
							{onDownload}
							{onRename}
							{onDelete}
							{onProperties}
						/>
					</div>
				{/each}
			</div>
		{/if}
	</div>
{/if}
