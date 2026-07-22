<script lang="ts">
	import Icon from './Icon.svelte';
	import type { User, FilerFile } from '$lib/types';
	import { clickOutside } from '$lib/actions/clickOutside';
	import { Button } from './ui/button/index.js';
	import { IconButton } from './ui/icon-button/index.js';
	import { Avatar } from './ui/avatar/index.js';
	import SearchBox from './SearchBox.svelte';
	import { Menu, MenuItem, MenuSeparator, MenuHeader } from './ui/menu/index.js';

	let {
		crumbs,
		searchValue = '',
		onPick,
		onSubmit,
		onUpload,
		dark,
		onToggleDark,
		onLogout,
		onSettings,
		user,
		searchRef: searchEl = $bindable()
	}: {
		crumbs: string[];
		searchValue?: string;
		onPick?: (file: FilerFile) => void;
		onSubmit?: (query: string) => void;
		onUpload?: () => void;
		dark: boolean;
		onToggleDark: () => void;
		onLogout: () => void;
		onSettings?: () => void;
		user: User;
		searchRef?: HTMLInputElement | null;
	} = $props();

	let menuOpen = $state(false);
</script>

<header class="flex h-12 shrink-0 items-center gap-3 border-b border-edge bg-surface px-4">
	<div class="flex items-center gap-1 text-[13.5px] whitespace-nowrap text-ink-muted">
		{#each crumbs as crumb, i}
			{#if i > 0}<Icon name="ChevronR" size={12} class="opacity-50" />{/if}
			{#if i === crumbs.length - 1}
				<b class="font-medium text-ink">{crumb}</b>
			{:else}
				<span>{crumb}</span>
			{/if}
		{/each}
	</div>

	{#if onSubmit && onPick}
		<SearchBox
			bind:ref={searchEl}
			class="ml-3 max-w-[360px] flex-1"
			initialValue={searchValue}
			{onPick}
			{onSubmit}
		/>
	{/if}

	<div class="ml-auto flex items-center gap-1.5">
		<IconButton title="Toggle theme" onclick={onToggleDark}>
			<Icon name={dark ? 'Sun' : 'Moon'} size={16} />
		</IconButton>

		{#if onUpload}
			<Button onclick={onUpload}>
				<Icon name="Upload" size={14} />
				Upload
			</Button>
		{/if}

		<div
			class="relative"
			use:clickOutside={{ enabled: menuOpen, onOutside: () => (menuOpen = false) }}
		>
			<IconButton class="p-0.5" onclick={() => (menuOpen = !menuOpen)} aria-label="Account menu">
				<Avatar name={user.name} />
			</IconButton>

			{#if menuOpen}
				<Menu class="top-[calc(100%+6px)] right-0">
					<MenuHeader title={user.name}>@{user.username}</MenuHeader>
					<MenuSeparator />
					<MenuItem
						onclick={() => {
							menuOpen = false;
							onSettings?.();
						}}
					>
						<Icon name="Lock" size={14} />
						Change password
					</MenuItem>
					<MenuItem
						onclick={() => {
							menuOpen = false;
							onSettings?.();
						}}
					>
						<Icon name="Settings" size={14} />
						Settings
					</MenuItem>
					<MenuSeparator />
					<MenuItem
						danger
						onclick={() => {
							menuOpen = false;
							onLogout();
						}}
					>
						<Icon name="LogOut" size={14} />
						Sign out
					</MenuItem>
				</Menu>
			{/if}
		</div>
	</div>
</header>
