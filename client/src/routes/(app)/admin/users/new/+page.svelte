<script lang="ts">
	import { goto } from '$app/navigation';
	import TopBar from '$lib/components/TopBar.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { createUser } from '$lib/services/users';
	import { Page } from '$lib/components/ui/page/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { BackButton } from '$lib/components/ui/back-button/index.js';
	import { Avatar } from '$lib/components/ui/avatar/index.js';
	import { Section } from '$lib/components/ui/section/index.js';
	import { Field } from '$lib/components/ui/field/index.js';
	import { Input, Select } from '$lib/components/ui/input/index.js';

	let name = $state('');
	let username = $state('');
	let role = $state<'user' | 'admin'>('user');
	let status = $state<'active' | 'suspended'>('active');
	let quotaGB = $state(20);
	let password = $state('');

	// Set once the server mints a password for us. While present we stay on the
	// page showing it, since it's the only time it can ever be read back.
	let tempPassword = $state<string | null>(null);
	let createdUsername = $state('');
	let copied = $state(false);

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		try {
			const created = await createUser({
				name,
				username,
				role,
				status,
				quotaGB,
				password: password || undefined
			});
			if (created.tempPassword) {
				// Hold on the page so the admin can copy the one-time password.
				tempPassword = created.tempPassword;
				createdUsername = username;
			} else {
				toastStore.show(`Created user @${username}`);
				goto('/admin/users');
			}
		} catch (err) {
			toastStore.show(err instanceof Error ? err.message : 'Failed to create user');
		}
	}

	function copyPassword() {
		if (!tempPassword) return;
		navigator.clipboard?.writeText(tempPassword).catch(() => {});
		copied = true;
		setTimeout(() => (copied = false), 1500);
	}
</script>

<TopBar
	crumbs={['Admin', 'Users', 'New']}
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

	<div class="mb-[22px] flex items-center gap-[14px]">
		<Avatar name={name || '? ?'} size="lg" />
		<div>
			<h1 class="m-0 text-[20px] font-semibold tracking-[-0.01em]">
				{tempPassword ? `Created @${createdUsername}` : 'New user'}
			</h1>
			<p class="m-0 text-[13px] text-ink-muted">
				{tempPassword
					? 'Share the temporary password below — it can’t be shown again.'
					: 'Create an account on this server'}
			</p>
		</div>
	</div>

	{#if tempPassword}
		<Section label="Temporary password">
			<p class="m-0 mb-3 text-[13px] text-ink-muted">
				This is the only time this password is shown. Copy it and share it with the user.
			</p>
			<div class="flex items-center gap-2">
				<code
					class="flex-1 select-all rounded-lg border border-edge bg-sunken px-3 py-2 font-mono text-[14px] tracking-wide text-ink"
					>{tempPassword}</code
				>
				<Button variant="ghost" onclick={copyPassword}>{copied ? 'Copied' : 'Copy'}</Button>
			</div>
			<div class="flex gap-2 pt-4">
				<Button onclick={() => goto('/admin/users')}>Done</Button>
			</div>
		</Section>
	{:else}
		<form onsubmit={handleSubmit} class="grid gap-[18px]">
		<Section label="Profile">
			<div class="flex gap-3">
				<Field label="Full name" class="mb-0 flex-1">
					<Input bind:value={name} required />
				</Field>
				<Field label="Username" class="mb-0 flex-1">
					<Input bind:value={username} required />
				</Field>
			</div>
		</Section>

		<Section label="Access">
			<div class="flex gap-3">
				<Field label="Role" hint="Admins can manage users and see all files." class="mb-0 flex-1">
					<Select bind:value={role}>
						<option value="user">User</option>
						<option value="admin">Admin</option>
					</Select>
				</Field>
				<Field label="Status" hint="Suspended accounts can't sign in." class="mb-0 flex-1">
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
				</div>
			</Field>
		</Section>

		<Section label="Initial password">
			<Field
				label="Password"
				hint="Leave blank to auto-generate."
				class="mb-0"
			>
				<Input type="text" placeholder="Auto-generate" bind:value={password} />
			</Field>
		</Section>

		<div class="flex gap-2 pt-1">
			<Button type="submit">Create user</Button>
			<Button variant="ghost" onclick={() => goto('/admin/users')}>Cancel</Button>
		</div>
	</form>
	{/if}
</Page>
