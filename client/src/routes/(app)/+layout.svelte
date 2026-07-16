<script lang="ts">
	import { goto } from '$app/navigation';
	import { browser } from '$app/environment';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Toast from '$lib/components/Toast.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { getFiles } from '$lib/services/files';
	import { getUsers } from '$lib/services/users';
	import type { FileSection, FilerFile, User } from '$lib/types';

	let { children } = $props();

	// Auth guard
	$effect(() => {
		if (browser && !authStore.isLoggedIn) {
			goto('/login', { replaceState: true });
		}
	});

	// Live counts for sidebar
	let allFiles = $state<FilerFile[]>([]);
	let allUsers = $state<User[]>([]);

	onMount(async () => {
		[allFiles, allUsers] = await Promise.all([getFiles(), getUsers()]);
	});

	const counts = $derived({
		my: allFiles.filter((f) => !f.trashed).length,
		public: allFiles.filter((f) => !f.trashed && f.public).length,
		trash: allFiles.filter((f) => f.trashed).length,
		users: allUsers.length
	});

	const usedGB = $derived(
		allFiles.filter((f) => !f.trashed).reduce((s, f) => s + f.size, 0) / 1_073_741_824
	);
	const quota = $derived({ used: usedGB, total: 500 });

	// Derive current screen + section from route
	const currentPath = $derived($page.url.pathname);

	const screen = $derived.by(() => {
		if (currentPath.startsWith('/admin/users') && currentPath !== '/admin/users') return 'admin-edit';
		if (currentPath === '/admin/users') return 'admin';
		if (currentPath === '/admin/audit') return 'audit';
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

	function handleSection(s: FileSection) {
		goto(`/files/${s}`);
	}

	function handleNav(n: string) {
		if (n === 'admin') goto('/admin/users');
		else if (n === 'audit') goto('/admin/audit');
	}
</script>

{#if authStore.user}
	<!-- .app — sidebar column collapses away when hidden -->
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
