<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { getUsers, updateUser, deleteUser } from '$lib/services/users';
	import type { User } from '$lib/types';
	import { Page } from '$lib/components/ui/page/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { BackButton } from '$lib/components/ui/back-button/index.js';
	import { Avatar } from '$lib/components/ui/avatar/index.js';
	import { Section } from '$lib/components/ui/section/index.js';
	import { Field } from '$lib/components/ui/field/index.js';
	import { Input, Select } from '$lib/components/ui/input/index.js';

	const userId = $derived($page.params.id ?? '');

	let user = $state<User | null>(null);
	let name = $state('');
	let username = $state('');
	let role = $state<'user' | 'admin'>('user');
	let status = $state<'active' | 'suspended'>('active');
	let quotaGB = $state(20);
	let password = $state('');

	onMount(async () => {
		const users = await getUsers();
		user = users.find((u) => u.id === userId) ?? null;
		if (user) {
			name = user.name;
			username = user.username;
			role = user.role;
			status = user.status ?? 'active';
			quotaGB = user.quotaGB ?? 0;
		}
	});

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		try {
			await updateUser(userId, { name, role, status, quotaGB, password: password || undefined });
			toastStore.show(`Saved changes to @${username}`);
			goto('/admin/users');
		} catch (err) {
			toastStore.show(err instanceof Error ? err.message : 'Failed to save changes');
		}
	}

	async function handleDelete() {
		if (!user) return;
		try {
			await deleteUser(user.id);
			toastStore.show(`Deleted user @${username}`);
			goto('/admin/users');
		} catch (err) {
			toastStore.show(err instanceof Error ? err.message : 'Failed to delete user');
		}
	}
</script>

<TopBar
	crumbs={['Admin', 'Users', name || '…']}
	dark={prefs.dark}
	onToggleDark={() => (prefs.dark = !prefs.dark)}
	user={authStore.user!}
	onLogout={() => {
		authStore.logout();
		goto('/login');
	}}
	onSettings={() => goto('/settings')}
/>

<Page class="max-w-[680px]">
	<div class="mb-[18px] flex flex-wrap items-baseline gap-3">
		<BackButton class="-ml-2" onclick={() => goto('/admin/users')}>Users</BackButton>
	</div>

	{#if !user}
		<p class="text-ink-muted">Loading…</p>
	{:else}
		<div class="mb-[22px] flex items-center gap-[14px]">
			<Avatar name={name || '? ?'} size="lg" />
			<div>
				<h1 class="m-0 text-[20px] font-semibold tracking-[-0.01em]">{name}</h1>
				<p class="m-0 text-[13px] text-ink-muted">Editing @{username}</p>
			</div>
		</div>

		<form onsubmit={handleSubmit} class="grid gap-[18px]">
			<Section label="Profile">
				<div class="flex gap-3">
					<Field label="Full name" class="mb-0 flex-1">
						<Input bind:value={name} required />
					</Field>
					<Field label="Username" hint="Contact your admin to change." class="mb-0 flex-1">
						<Input value={username} disabled />
					</Field>
				</div>
			</Section>

			<Section label="Access">
				<div class="flex gap-3">
					<Field label="Role" class="mb-0 flex-1">
						<Select bind:value={role}>
							<option value="user">User</option>
							<option value="admin">Admin</option>
						</Select>
					</Field>
					<Field label="Status" class="mb-0 flex-1">
						<Select bind:value={status}>
							<option value="active">Active</option>
							<option value="suspended">Suspended</option>
						</Select>
					</Field>
				</div>
				<Field label="Storage quota" class="mb-0">
					<div class="flex items-center gap-3">
						<Input type="number" min={1} max={2000} bind:value={quotaGB} class="w-[110px]" />
						<span class="text-ink-muted">GB</span>
						<span class="ml-auto text-[12px] text-ink-muted">
							Currently using {(user.usedGB ?? 0).toFixed(1)} GB
						</span>
					</div>
				</Field>
			</Section>

			<Section label="Reset password">
				<Field label="New password" class="mb-0">
					<Input type="text" placeholder="Leave blank to keep current" bind:value={password} />
				</Field>
			</Section>

			<div class="flex gap-2 pt-1">
				<Button type="submit">Save changes</Button>
				<Button variant="ghost" onclick={() => goto('/admin/users')}>Cancel</Button>
				<Button variant="ghost" class="ml-auto border-transparent text-danger" onclick={handleDelete}>
					<Icon name="Trash" size={14} />
					Delete account
				</Button>
			</div>
		</form>
	{/if}
</Page>
