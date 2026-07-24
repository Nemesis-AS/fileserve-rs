<script lang="ts">
	import { goto } from '$app/navigation';
	import { browser } from '$app/environment';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Toast from '$lib/components/Toast.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { quotaStore } from '$lib/stores/quota.svelte';
	import { getFiles, getPublicFiles } from '$lib/services/files';
	import { getUsers, getMe } from '$lib/services/users';
	import type { FileSection, FilerFile, User } from '$lib/types';

	let { children } = $props();

	let hydrated = $state(false);

	$effect(() => {
		if (browser && hydrated && !authStore.isLoggedIn) {
			goto('/login', { replaceState: true });
		}
	});

	let allFiles = $state<FilerFile[]>([]);
	let publicFiles = $state<FilerFile[]>([]);
	let allUsers = $state<User[]>([]);

	onMount(async () => {
		try {
			authStore.login(await getMe());
		} catch {
			// Keep whatever the store already had (if anything).
		} finally {
			hydrated = true;
		}

		[allFiles, publicFiles, allUsers] = await Promise.all([
			getFiles().catch(() => []),
			getPublicFiles().catch(() => []),
			getUsers().catch(() => [])
		]);
	});

	const counts = $derived({
		my: allFiles.filter((f) => !f.trashed).length,
		public: publicFiles.length,
		trash: allFiles.filter((f) => f.trashed).length,
		users: allUsers.length
	});

	$effect(() => {
		void $page.url.pathname;
		void quotaStore.refresh();
	});

	const quota = $derived({ used: quotaStore.quota.usedGB, total: quotaStore.quota.quotaGB });

	const currentPath = $derived($page.url.pathname);

	const screen = $derived.by(() => {
		if (currentPath.startsWith('/admin/users') && currentPath !== '/admin/users') return 'admin-edit';
		if (currentPath === '/admin/users') return 'admin';
		if (currentPath === '/admin/audit') return 'audit';
		if (currentPath === '/admin/config') return 'config';
		if (currentPath === '/settings') return 'settings';
		if (currentPath.match(/^\/files\/[^/]+\/[^/]+/)) return 'viewer';
		return 'files';
	});

	const section = $derived.by((): FileSection => {
		if (currentPath.includes('/files/public')) return 'public';
		if (currentPath.includes('/files/trash')) return 'trash';
		return 'my';
	});

	const isAdmin = $derived(authStore.user?.role === 'admin');

	const pageTitle = $derived.by(() => {
		if (screen === 'admin' || screen === 'admin-edit') return 'Users';
		if (screen === 'audit') return 'Audit log';
		if (screen === 'config') return 'Configuration';
		if (screen === 'settings') return 'Settings';
		if (section === 'public') return 'Public';
		if (section === 'trash') return 'Trash';
		return 'My Files';
	});

	function handleSection(s: FileSection) {
		goto(`/files/${s}`);
	}

	function handleNav(n: string) {
		if (n === 'admin') goto('/admin/users');
		else if (n === 'audit') goto('/admin/audit');
		else if (n === 'config') goto('/admin/config');
	}
</script>

<svelte:head>
	<title>{pageTitle} · fileserve.rs</title>
</svelte:head>

{#if authStore.user}
	<div
		class="grid h-full overflow-hidden bg-surface {prefs.showSidebar
			? 'grid-cols-[auto_1fr]'
			: 'grid-cols-[1fr]'}"
	>
		{#if prefs.showSidebar}
			<Sidebar
				{section}
				{screen}
				{isAdmin}
				{counts}
				{quota}
				onSection={handleSection}
				onNav={handleNav}
			/>
		{/if}

		<main class="relative flex min-h-0 min-w-0 flex-col overflow-hidden">
			{@render children()}
		</main>
	</div>

	<Toast />
{/if}
