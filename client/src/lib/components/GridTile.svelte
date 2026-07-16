<script lang="ts">
	import Icon from './Icon.svelte';
	import type { FilerFile, FileSection } from '$lib/types';
	import { fmtSize, fmtDate } from '$lib/utils/file';
	import { clickOutside } from '$lib/actions/clickOutside';
	import { IconButton } from './ui/icon-button/index.js';
	import { Menu, MenuItem, MenuSeparator } from './ui/menu/index.js';

	let {
		file,
		active,
		section,
		onclick,
		ondblclick,
		onView,
		onDownload,
		onRename,
		onDelete,
		onProperties
	}: {
		file: FilerFile;
		active: boolean;
		section: FileSection;
		onclick: () => void;
		ondblclick: () => void;
		onView: (f: FilerFile) => void;
		onDownload: (f: FilerFile) => void;
		onRename: (f: FilerFile) => void;
		onDelete: (f: FilerFile) => void;
		onProperties: (f: FilerFile) => void;
	} = $props();

	let menuOpen = $state(false);

	function fire(fn: (f: FilerFile) => void) {
		return (e: MouseEvent) => {
			e.stopPropagation();
			menuOpen = false;
			fn(file);
		};
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!--
  `group/tile` drives the action-bar reveal: the actions are hidden until the tile is
  hovered, selected, or has its menu open — the old `.gridtile:hover .gridtile__actions`
  + `[data-active]` + `[data-menu-open]` rules.
-->
<div
	class="group/tile relative flex cursor-default flex-col overflow-hidden rounded-[10px] border border-edge bg-surface transition-[border-color,box-shadow] duration-[120ms] hover:border-edge-strong data-[active=1]:border-accent data-[active=1]:shadow-[0_0_0_1px_var(--accent)]"
	data-active={active ? '1' : '0'}
	data-menu-open={menuOpen ? '1' : '0'}
	{onclick}
	{ondblclick}
>
	<div
		class="relative grid aspect-[4/3] place-items-center overflow-hidden bg-sunken bg-cover bg-center"
		data-thumb={file.thumb ? '1' : '0'}
		style={file.thumb ? `background-image: url(${file.thumb})` : undefined}
	>
		{#if !file.thumb}
			<div
				class="rounded-md px-2.5 py-1.5 font-code text-[13px] font-semibold tracking-normal text-white"
				style="background: {file.color};"
			>
				{file.ext.toUpperCase()}
			</div>
		{/if}

		{#if file.public}
			<span
				class="absolute top-2 right-2 rounded-[3px] bg-black/55 px-1.5 py-0.5 text-[9.5px] font-semibold tracking-[0.05em] text-white uppercase backdrop-blur-[8px]"
			>
				Shared
			</span>
		{/if}

		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="absolute top-1.5 left-1.5 flex gap-1 opacity-0 transition-opacity duration-[120ms] group-hover/tile:opacity-100 group-data-[active=1]/tile:opacity-100 group-data-[menu-open=1]/tile:opacity-100"
			onclick={(e) => e.stopPropagation()}
		>
			<IconButton variant="overlay" size="sm" title="Download" onclick={fire(onDownload)}>
				<Icon name="Download" size={14} />
			</IconButton>
			<IconButton
				variant="overlay"
				size="sm"
				title="More options"
				onclick={(e) => {
					e.stopPropagation();
					menuOpen = !menuOpen;
				}}
			>
				<Icon name="More" size={14} />
			</IconButton>
		</div>

		{#if menuOpen}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				use:clickOutside={{ onOutside: () => (menuOpen = false) }}
				onclick={(e) => e.stopPropagation()}
			>
				<Menu size="compact" class="top-[38px] left-1.5" flyY={-4} flyDuration={120}>
					<MenuItem onclick={fire(onView)}>
						<Icon name="Eye" size={14} />Open
					</MenuItem>
					<MenuItem onclick={fire(onDownload)}>
						<Icon name="Download" size={14} />Download
					</MenuItem>
					{#if section !== 'trash'}
						<MenuItem onclick={fire(onRename)}>
							<Icon name="Pencil" size={14} />Rename
						</MenuItem>
					{/if}
					<MenuItem onclick={fire(onProperties)}>
						<Icon name="Info" size={14} />Properties
					</MenuItem>
					<MenuSeparator />
					<MenuItem danger onclick={fire(onDelete)}>
						<Icon name="Trash" size={14} />
						{section === 'trash' ? 'Delete permanently' : 'Move to Trash'}
					</MenuItem>
				</Menu>
			</div>
		{/if}
	</div>

	<div class="flex flex-col gap-0.5 border-t border-edge px-[11px] pt-[9px] pb-[11px]">
		<div class="overflow-hidden text-[13px] font-medium text-ellipsis whitespace-nowrap">
			{file.name}
		</div>
		<div class="flex justify-between gap-1.5 text-[11.5px] text-ink-muted tabular-nums">
			<span>{fmtSize(file.size)}</span>
			<span>{fmtDate(file.modified)}</span>
		</div>
	</div>
</div>
