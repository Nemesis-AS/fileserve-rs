<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { getAuditEvents } from '$lib/services/audit';
	import { fmtDate } from '$lib/utils/file';
	import type { AuditEvent, AuditEventType } from '$lib/types';
	import { Page, PageHead } from '$lib/components/ui/page/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { SearchInput } from '$lib/components/ui/search-input/index.js';
	import { Select } from '$lib/components/ui/input/index.js';
	import { tv } from '$lib/components/ui/tv.js';

	let events = $state<AuditEvent[]>([]);
	let filterType = $state('all');
	let filterUser = $state('all');
	let search = $state('');

	onMount(async () => {
		events = await getAuditEvents();
	});

	const EVENT_META: Record<AuditEventType, { label: string; tone: string; icon: string }> = {
		upload:    { label: 'Upload',         tone: 'info',    icon: 'Upload' },
		download:  { label: 'Download',       tone: 'neutral', icon: 'Download' },
		rename:    { label: 'Rename',         tone: 'neutral', icon: 'Pencil' },
		delete:    { label: 'Delete',         tone: 'warn',    icon: 'Trash' },
		restore:   { label: 'Restore',        tone: 'success', icon: 'Refresh' },
		share:     { label: 'Share',          tone: 'info',    icon: 'Link' },
		login:     { label: 'Sign in',        tone: 'neutral', icon: 'LogOut' },
		auth_fail: { label: 'Failed sign-in', tone: 'danger',  icon: 'Lock' },
		user_add:  { label: 'User created',   tone: 'info',    icon: 'Plus' },
		user_edit: { label: 'User changed',   tone: 'neutral', icon: 'Users' },
		password:  { label: 'Password',       tone: 'neutral', icon: 'Lock' },
	};

	const VERBS: Record<AuditEventType, string> = {
		upload: 'uploaded', download: 'downloaded', rename: 'renamed', delete: 'deleted',
		restore: 'restored', share: 'shared', login: 'signed in', auth_fail: 'failed to sign in',
		user_add: 'created', user_edit: 'updated', password: 'changed password',
	};

	/** `.audit-row__icon[data-tone]` — the tinted plate at the head of each row. */
	const rowIcon = tv({
		base: 'grid size-7 shrink-0 place-items-center rounded-[7px] bg-sunken text-ink-muted [&_svg]:size-[14px]',
		variants: {
			tone: {
				neutral: '',
				info: 'bg-accent-soft text-accent-ink',
				warn: 'bg-[rgba(217,119,6,0.1)] text-warn dark:bg-[rgba(251,191,36,0.1)]',
				danger: 'bg-danger-soft text-danger',
				success: 'bg-[rgba(22,163,74,0.1)] text-ok'
			}
		},
		defaultVariants: { tone: 'neutral' }
	});

	const allUsers = $derived([...new Set(events.map((e) => e.actor))]);
	const allTypes = $derived([...new Set(events.map((e) => e.type))]);

	const filtered = $derived.by(() =>
		events.filter((e) => {
			if (filterType !== 'all' && e.type !== filterType) return false;
			if (filterUser !== 'all' && e.actor !== filterUser) return false;
			if (search) {
				const q = search.toLowerCase();
				const hay = `${e.actor} ${e.target ?? ''} ${e.meta ?? ''} ${e.ip}`.toLowerCase();
				if (!hay.includes(q)) return false;
			}
			return true;
		})
	);
</script>

<TopBar
	crumbs={['Admin', 'Audit log']}
	dark={prefs.dark}
	onToggleDark={() => (prefs.dark = !prefs.dark)}
	user={authStore.user!}
	onLogout={() => {
		authStore.logout();
		goto('/login');
	}}
	onSettings={() => goto('/settings')}
/>

<Page>
	<PageHead title="Audit log" sub="{filtered.length} events · last 48 hours">
		{#snippet actions()}
			<Button variant="ghost" onclick={() => toastStore.show('Exporting CSV…')}>
				<Icon name="Download" size={14} />
				Export CSV
			</Button>
		{/snippet}
	</PageHead>

	<!-- .audit-filters -->
	<div class="mb-[14px] flex flex-wrap items-center gap-2">
		<SearchInput
			class="max-w-[280px] flex-1"
			placeholder="Filter by user, file, IP…"
			bind:value={search}
		/>
		<Select variant="filter" bind:value={filterType}>
			<option value="all">All events</option>
			{#each allTypes as t}
				<option value={t}>{EVENT_META[t]?.label ?? t}</option>
			{/each}
		</Select>
		<Select variant="filter" bind:value={filterUser}>
			<option value="all">All users</option>
			{#each allUsers as u}
				<option value={u}>@{u}</option>
			{/each}
		</Select>
	</div>

	<!-- .audit -->
	<ul class="m-0 list-none overflow-hidden rounded-[10px] border border-edge bg-surface p-0">
		{#each filtered as e (e.id)}
			{@const meta = EVENT_META[e.type] ?? { label: e.type, tone: 'neutral', icon: 'Info' }}
			<li
				class="flex items-start gap-3 border-b border-edge px-[14px] py-3 transition-colors duration-[80ms] last:border-b-0 hover:bg-row-hover"
			>
				<div class={rowIcon({ tone: meta.tone as 'info' | 'warn' | 'danger' | 'success' | 'neutral' })}>
					<Icon name={meta.icon as never} size={14} />
				</div>
				<div class="flex min-w-0 flex-1 flex-col gap-1">
					<div class="overflow-hidden text-[13px] text-ellipsis whitespace-nowrap">
						<b class="font-code text-[12.5px] font-medium">@{e.actor}</b>
						<span class="text-ink-muted"> {VERBS[e.type] ?? e.type} </span>
						{#if e.target}
							<span class="rounded bg-sunken px-1.5 py-px font-code text-[12px] text-ink">
								{e.target}
							</span>
						{/if}
						{#if e.meta}<span class="ml-1 text-[12.5px] text-ink-faint">· {e.meta}</span>{/if}
					</div>
					<!-- .audit-row__sub — the badge here is squatter than the default -->
					<div class="flex flex-wrap items-center gap-2 text-[11.5px] text-ink-muted [&>*]:whitespace-nowrap">
						<Badge
							tone={meta.tone === 'danger' ? 'suspended' : 'neutral'}
							class="h-[17px] px-1.5 py-0 text-[10.5px] leading-4"
						>
							{meta.label}
						</Badge>
						<span>{fmtDate(e.at)}</span>
						<span class="font-code">{e.ip}</span>
					</div>
				</div>
			</li>
		{/each}
		{#if filtered.length === 0}
			<li class="p-10 text-center text-ink-muted">No matching events.</li>
		{/if}
	</ul>
</Page>
