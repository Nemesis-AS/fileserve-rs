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
	let email = $state('');
	let role = $state<'user' | 'admin'>('user');
	let status = $state<'active' | 'suspended'>('active');
	let quotaGB = $state(50);
	let password = $state('');

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		await createUser({ name, username, email, role, status, quotaGB });
		toastStore.show(`Created user @${username}`);
		goto('/admin/users');
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
			<h1 class="m-0 text-[20px] font-semibold tracking-[-0.01em]">New user</h1>
			<p class="m-0 text-[13px] text-ink-muted">Create an account on this server</p>
		</div>
	</div>

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
			<Field label="Email" hint="Used for password resets only." class="mb-0">
				<Input type="email" bind:value={email} />
			</Field>
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
				hint="Leave blank to auto-generate. The user will be prompted to change it on first login."
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
</Page>
