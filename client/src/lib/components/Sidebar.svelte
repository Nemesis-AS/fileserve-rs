<script lang="ts">
	import Icon from './Icon.svelte';
	import Logo from './Logo.svelte';
	import type { FileSection } from '$lib/types';
	import { NavItem } from './ui/nav-item/index.js';
	import { Meter } from './ui/meter/index.js';

	let {
		section,
		screen,
		isAdmin,
		counts,
		quota,
		onSection,
		onNav
	}: {
		section: FileSection;
		screen: string;
		isAdmin: boolean;
		counts: { my: number; public: number; trash: number; users: number };
		/** `total` in GB; null means no quota limit. */
		quota: { used: number; total: number | null };
		onSection: (s: FileSection) => void;
		onNav: (n: string) => void;
	} = $props();

	const pct = $derived(
		quota.total && quota.total > 0 ? Math.min(100, (quota.used / quota.total) * 100) : 0
	);
</script>

<aside class="flex min-h-0 w-58 flex-col overflow-hidden border-r border-edge bg-elevated">
	<div class="flex shrink-0 items-center gap-2.25 px-4 pt-3.5 pb-3">
		<Logo size={22} />
		<span class="text-[15px] font-bold tracking-[-0.035em] text-ink">
			fileserve<span class="mx-[1.5px] inline-block size-1 rounded-full bg-accent align-baseline"
			></span><span class="font-mono text-[13px] font-semibold text-accent-ink">rs</span>
		</span>
	</div>

	<nav class="scroll-area flex flex-1 flex-col gap-px overflow-y-auto px-2 py-1">
		<NavItem
			active={screen === 'files' && section === 'my'}
			count={counts.my}
			onclick={() => onSection('my')}
		>
			{#snippet icon()}<Icon name="Folder" size={16} />{/snippet}
			My Files
		</NavItem>

		<NavItem
			active={screen === 'files' && section === 'public'}
			count={counts.public}
			onclick={() => onSection('public')}
		>
			{#snippet icon()}<Icon name="Public" size={16} />{/snippet}
			Public
		</NavItem>

		<NavItem
			active={screen === 'files' && section === 'trash'}
			count={counts.trash > 0 ? counts.trash : undefined}
			onclick={() => onSection('trash')}
		>
			{#snippet icon()}<Icon name="TrashBin" size={16} />{/snippet}
			Trash
		</NavItem>

		{#if isAdmin}
			<div
				class="px-2.5 pt-3.5 pb-1.5 text-[10.5px] font-semibold tracking-[0.06em] whitespace-nowrap text-ink-faint uppercase"
			>
				Admin
			</div>

			<NavItem
				active={screen === 'admin' || screen === 'admin-edit'}
				count={counts.users}
				onclick={() => onNav('admin')}
			>
				{#snippet icon()}<Icon name="Users" size={16} />{/snippet}
				Users
			</NavItem>

			<NavItem active={screen === 'audit'} onclick={() => onNav('audit')}>
				{#snippet icon()}<Icon name="Info" size={16} />{/snippet}
				Audit log
			</NavItem>

			<NavItem active={screen === 'config'} onclick={() => onNav('config')}>
				{#snippet icon()}<Icon name="Settings" size={16} />{/snippet}
				Configuration
			</NavItem>
		{/if}
	</nav>

	<div
		class="flex shrink-0 flex-col gap-1.5 border-t border-edge px-4 pt-3.5 pb-4 text-[12px] text-ink-muted"
	>
		<div class="flex justify-between">
			<span>Storage</span>
			{#if quota.total == null}
				<span><b class="font-medium text-ink">{quota.used.toFixed(1)}</b> GB used</span>
			{:else}
				<span><b class="font-medium text-ink">{quota.used.toFixed(1)}</b> / {quota.total} GB</span>
			{/if}
		</div>
		{#if quota.total == null}
			<span class="text-[11px] text-ink-faint">No quota limit</span>
		{:else}
			<Meter value={pct} />
			<div class="flex justify-between text-[11px]">
				<span class="text-ink-faint">{Math.max(0, quota.total - quota.used).toFixed(1)} GB free</span>
				<span class="text-ink-faint">{Math.round(pct)}%</span>
			</div>
		{/if}
	</div>
</aside>
