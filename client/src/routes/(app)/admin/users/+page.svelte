<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { getUsers } from '$lib/services/users';
	import { DEMO_USERS } from '$lib/mock/demoAdmin';
	import { fmtGB } from '$lib/utils/file';
	import type { User } from '$lib/types';
	import { Page, PageHead } from '$lib/components/ui/page/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Avatar } from '$lib/components/ui/avatar/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Meter } from '$lib/components/ui/meter/index.js';
	import { table } from '$lib/components/ui/table/index.js';

	let users = $state<User[]>([]);

	const isDemo = $derived(authStore.user?.demo ?? false);

	onMount(async () => {
		// A demo visitor never calls the admin API: the server would refuse it,
		// and not asking means there is no path by which real accounts could
		// reach the page. The roster below is openly fabricated.
		if (authStore.user?.demo) {
			users = DEMO_USERS;
			return;
		}
		try {
			users = await getUsers();
		} catch (e) {
			toastStore.show(e instanceof Error ? e.message : 'Failed to load users');
		}
	});

	const t = table({ variant: 'users' });
</script>

<TopBar
	crumbs={['Admin', 'Users']}
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
	<PageHead
		title="Users"
		sub={isDemo ? 'Sample data · read-only in the demo' : `${users.length} accounts · admin-managed`}
	>
		{#snippet actions()}
			{#if !isDemo}
				<Button onclick={() => goto('/admin/users/new')}>
					<Icon name="Plus" size={14} />
					Add user
				</Button>
			{/if}
		{/snippet}
	</PageHead>

	<table class={t.root()}>
		<thead>
			<tr>
				<th class={t.th()}>User</th>
				<th class={t.th({ class: 'w-[90px]' })}>Role</th>
				<th class={t.th({ class: 'w-[110px]' })}>Status</th>
				<th class={t.th({ class: 'w-[200px]' })}>Storage</th>
				<th class={t.th({ class: 'w-[80px]' })}>Files</th>
				<th class={t.th({ class: 'w-[30px]' })}></th>
			</tr>
		</thead>
		<tbody>
			{#each users as u (u.id)}
				{@const used = u.usedGB ?? 0}
				{@const quota = u.quotaGB}
				{@const pct = quota && quota > 0 ? Math.min(100, (used / quota) * 100) : 0}
				<tr class={t.tr({ class: 'cursor-pointer' })} onclick={() => goto(`/admin/users/${u.id}`)}>
					<td class={t.td()}>
						<div class="flex items-center gap-2.5">
							<Avatar name={u.name} size="md" />
							<div>
								<div class="font-medium">{u.name}</div>
								<div class="text-[11.5px] text-ink-muted">@{u.username}</div>
							</div>
						</div>
					</td>
					<td class={t.td()}>
						<Badge tone={u.role === 'admin' ? 'admin' : 'neutral'}>
							{u.role === 'admin' ? 'Admin' : 'User'}
						</Badge>
					</td>
					<td class={t.td()}>
						<Badge tone={u.status ?? 'active'} dot>
							{u.status === 'suspended' ? 'Suspended' : 'Active'}
						</Badge>
					</td>
					<td class={t.td()}>
						<!-- .user-quota-mini — a slimmer quota bar than the sidebar's -->
						<div class="flex min-w-[140px] flex-col gap-1">
							{#if quota == null}
								<span class="text-[11.5px] text-ink-muted tabular-nums">
									{fmtGB(used)} · no limit
								</span>
							{:else}
								<Meter value={pct} size="xs" />
								<span class="text-[11.5px] text-ink-muted tabular-nums">
									{fmtGB(used)} / {fmtGB(quota)}
								</span>
							{/if}
						</div>
					</td>
					<td class={t.td({ class: 'text-ink-muted tabular-nums' })}>{u.files ?? 0}</td>
					<td class={t.td()}><Icon name="ChevronR" size={16} class="text-ink-faint" /></td>
				</tr>
			{/each}
		</tbody>
	</table>
</Page>
