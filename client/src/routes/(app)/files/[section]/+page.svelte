<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import FileBrowser from '$lib/components/FileBrowser.svelte';
	import UploadModal from '$lib/components/UploadModal.svelte';
	import PropertiesModal from '$lib/components/PropertiesModal.svelte';
	import RenameModal from '$lib/components/RenameModal.svelte';
	import DeleteConfirm from '$lib/components/DeleteConfirm.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { getFiles, trashFile, deleteFile, renameFile, toggleShare, filterBySection } from '$lib/services/files';
	import type { FilerFile, FileSection } from '$lib/types';

	const sectionParam = $derived($page.params.section as FileSection);

	let allFiles = $state<FilerFile[]>([]);
	let activeId = $state<string | null>(null);
	let search = $state('');
	let showUpload = $state(false);
	let propsFile = $state<FilerFile | null>(null);
	let renameTarget = $state<FilerFile | null>(null);
	let deleteTarget = $state<FilerFile | null>(null);
	let searchEl = $state<HTMLInputElement | null>(null);

	onMount(async () => {
		allFiles = await getFiles();
	});

	const sectionFiles = $derived.by(() => {
		const list = filterBySection(allFiles, sectionParam);
		if (!search) return list;
		const q = search.toLowerCase();
		return list.filter((f) => f.name.toLowerCase().includes(q));
	});

	const TITLES: Record<FileSection, string> = {
		my: 'My Files', public: 'Public', trash: 'Trash'
	};

	// ── Actions ──────────────────────────────────────────────
	function handleView(f: FilerFile) {
		goto(`/files/${sectionParam}/${f.id}`);
	}

	function handleDownload(f: FilerFile) {
		toastStore.show(`Downloading "${f.name}"`);
	}

	async function handleDelete(f: FilerFile) {
		deleteTarget = null;
		if (sectionParam === 'trash') {
			await deleteFile(f.id);
			allFiles = allFiles.filter((x) => x.id !== f.id);
			toastStore.show(`Deleted "${f.name}" permanently`);
		} else {
			await trashFile(f.id);
			const prev = { ...f };
			allFiles = allFiles.map((x) => x.id === f.id ? { ...x, trashed: true, trashedAt: new Date().toISOString() } : x);
			toastStore.show(`Moved "${f.name}" to Trash`, () => {
				allFiles = allFiles.map((x) => x.id === f.id ? { ...x, trashed: false, trashedAt: null } : x);
			});
		}
	}

	async function handleRename(f: FilerFile, newName: string) {
		renameTarget = null;
		const updated = await renameFile(f.id, newName);
		allFiles = allFiles.map((x) => x.id === f.id ? updated : x);
	}

	async function handleToggleShare(f: FilerFile, makePublic: boolean) {
		const updated = await toggleShare(f.id, makePublic);
		allFiles = allFiles.map((x) => x.id === f.id ? updated : x);
		if (propsFile?.id === f.id) propsFile = updated;
	}

	// ── Keyboard shortcuts ───────────────────────────────────
	function handleKeydown(e: KeyboardEvent) {
		const tag = (e.target as HTMLElement).tagName?.toLowerCase();
		const inText = tag === 'input' || tag === 'textarea' || (e.target as HTMLElement).isContentEditable;

		if (e.key === '/' && !inText) {
			e.preventDefault();
			searchEl?.focus();
			searchEl?.select();
			return;
		}
		if (inText) return;

		if (e.key === 'Escape' && search) { search = ''; return; }

		if (e.key === 'j' || e.key === 'ArrowDown' || e.key === 'k' || e.key === 'ArrowUp') {
			const ids = sectionFiles.map((f) => f.id);
			if (!ids.length) return;
			const idx = ids.indexOf(activeId ?? '');
			const next = (e.key === 'j' || e.key === 'ArrowDown')
				? (idx < 0 ? 0 : Math.min(ids.length - 1, idx + 1))
				: (idx < 0 ? 0 : Math.max(0, idx - 1));
			activeId = ids[next];
			e.preventDefault();
		}
		if ((e.key === 'Enter' || e.key === 'o') && activeId) {
			const f = sectionFiles.find((x) => x.id === activeId);
			if (f) handleView(f);
			e.preventDefault();
		}
		if ((e.key === 'Delete' || e.key === 'Backspace') && activeId && sectionParam !== 'trash') {
			const f = sectionFiles.find((x) => x.id === activeId);
			if (f) deleteTarget = f;
			e.preventDefault();
		}
		if (e.key === 'r' && activeId) {
			const f = sectionFiles.find((x) => x.id === activeId);
			if (f) renameTarget = f;
			e.preventDefault();
		}
		if ((e.key === 'i' || e.key === ' ') && activeId) {
			const f = sectionFiles.find((x) => x.id === activeId);
			if (f) propsFile = f;
			e.preventDefault();
		}
		if (e.key === 'u') {
			showUpload = true;
			e.preventDefault();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<TopBar
	crumbs={['filer', TITLES[sectionParam] ?? 'Files']}
	{search}
	onSearch={(v) => { search = v; }}
	onUpload={() => (showUpload = true)}
	dark={prefs.dark}
	onToggleDark={() => (prefs.dark = !prefs.dark)}
	user={authStore.user!}
	onLogout={() => { authStore.logout(); goto('/login'); }}
	onSettings={() => goto('/settings')}
	bind:searchRef={searchEl}
/>

<FileBrowser
	files={sectionFiles}
	view={prefs.view}
	section={sectionParam}
	{search}
	{activeId}
	onActiveId={(id) => (activeId = id)}
	onViewChange={(v) => (prefs.view = v)}
	onView={handleView}
	onDownload={handleDownload}
	onRename={(f) => (renameTarget = f)}
	onDelete={(f) => (deleteTarget = f)}
	onProperties={(f) => (propsFile = f)}
	onUpload={() => (showUpload = true)}
/>

{#if showUpload}
	<UploadModal onClose={() => (showUpload = false)} />
{/if}

{#if propsFile}
	<PropertiesModal
		file={propsFile}
		onClose={() => (propsFile = null)}
		onToggleShare={handleToggleShare}
	/>
{/if}

{#if renameTarget}
	<RenameModal
		file={renameTarget}
		onClose={() => (renameTarget = null)}
		onRename={(name) => handleRename(renameTarget!, name)}
	/>
{/if}

{#if deleteTarget}
	<DeleteConfirm
		file={deleteTarget}
		inTrash={sectionParam === 'trash'}
		onClose={() => (deleteTarget = null)}
		onConfirm={() => handleDelete(deleteTarget!)}
	/>
{/if}
