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
	import { quotaStore } from '$lib/stores/quota.svelte';
	import {
		getFiles,
		getPublicFiles,
		searchFiles,
		trashFile,
		restoreFile,
		deleteFile,
		renameFile,
		downloadFile,
		toggleShare,
		filterBySection
	} from '$lib/services/files';
	import type { FilerFile, FileSection } from '$lib/types';

	const sectionParam = $derived($page.params.section as FileSection);
	// A `?q=` param turns the section into a server-backed search results view.
	const query = $derived($page.url.searchParams.get('q'));

	let allFiles = $state<FilerFile[]>([]);
	// The Public section is a cross-owner listing, not a filter over `allFiles`.
	let publicFiles = $state<FilerFile[]>([]);
	let searchResults = $state<FilerFile[]>([]);
	let activeId = $state<string | null>(null);
	let showUpload = $state(false);
	let propsFile = $state<FilerFile | null>(null);
	let renameTarget = $state<FilerFile | null>(null);
	let deleteTarget = $state<FilerFile | null>(null);
	let searchEl = $state<HTMLInputElement | null>(null);

	async function refresh() {
		try {
			allFiles = await getFiles();
			// Usage changed (e.g. an upload just completed) — keep the sidebar bar
			// in step rather than waiting for the next navigation.
			void quotaStore.refresh();
		} catch (e) {
			toastStore.show(e instanceof Error ? e.message : 'Could not load files');
		}
	}

	onMount(refresh);

	// The Public section is a separate cross-owner fetch; (re)load it on entry so
	// a file another user just made public shows up without a full reload.
	$effect(() => {
		if (sectionParam !== 'public') return;
		let active = true;
		getPublicFiles()
			.then((f) => active && (publicFiles = f))
			.catch((e) => active && toastStore.show(e instanceof Error ? e.message : 'Could not load public files'));
		return () => {
			active = false;
		};
	});

	// Fetch results whenever `?q=` changes; a flag drops stale in-flight lookups.
	$effect(() => {
		const q = query;
		if (!q) {
			searchResults = [];
			return;
		}
		let active = true;
		searchFiles(q)
			.then((r) => active && (searchResults = r))
			.catch((e) => {
				if (!active) return;
				searchResults = [];
				toastStore.show(e instanceof Error ? e.message : 'Search failed');
			});
		return () => {
			active = false;
		};
	});

	const displayFiles = $derived.by(() => {
		if (query) return searchResults.filter((f) => !f.trashed);
		// Server already scopes this to non-trashed public rows across all owners.
		if (sectionParam === 'public') return publicFiles;
		return filterBySection(allFiles, sectionParam);
	});

	const TITLES: Record<FileSection, string> = {
		my: 'My Files', public: 'Public', trash: 'Trash'
	};

	// ── Actions ──────────────────────────────────────────────
	function handleView(f: FilerFile) {
		goto(`/files/${sectionParam}/${f.id}`);
	}

	function handleSearchSubmit(q: string) {
		goto(`/files/${sectionParam}?q=${encodeURIComponent(q)}`);
	}

	function handleDownload(f: FilerFile) {
		downloadFile(f);
	}

	function replace(updated: FilerFile) {
		allFiles = allFiles.map((x) => (x.id === updated.id ? updated : x));
		searchResults = searchResults.map((x) => (x.id === updated.id ? updated : x));
	}

	/** Surfaces the server's reason in a toast instead of failing silently. */
	function fail(e: unknown, fallback: string) {
		toastStore.show(e instanceof Error ? e.message : fallback);
	}

	async function handleDelete(f: FilerFile) {
		deleteTarget = null;
		try {
			if (sectionParam === 'trash') {
				await deleteFile(f.id);
				allFiles = allFiles.filter((x) => x.id !== f.id);
				searchResults = searchResults.filter((x) => x.id !== f.id);
				toastStore.show(`Deleted "${f.name}" permanently`);
			} else {
				replace(await trashFile(f.id));
				// Undo restores server-side too, so the row survives a reload.
				toastStore.show(`Moved "${f.name}" to Trash`, async () => {
					try {
						replace(await restoreFile(f.id));
						void quotaStore.refresh();
					} catch (e) {
						fail(e, 'Could not restore file');
					}
				});
			}
			// Trashing and permanent deletion both change counted usage.
			void quotaStore.refresh();
		} catch (e) {
			fail(e, 'Could not delete file');
		}
	}

	async function handleRestore(f: FilerFile) {
		try {
			replace(await restoreFile(f.id));
			void quotaStore.refresh(); // restoring re-counts the file against usage
			toastStore.show(`Restored "${f.name}"`);
		} catch (e) {
			fail(e, 'Could not restore file');
		}
	}

	async function handleRename(f: FilerFile, newName: string) {
		renameTarget = null;
		try {
			replace(await renameFile(f.id, newName));
		} catch (e) {
			fail(e, 'Could not rename file');
		}
	}

	async function handleToggleShare(f: FilerFile, makePublic: boolean) {
		try {
			const updated = await toggleShare(f, makePublic);
			replace(updated);
			// Keep the Public listing in step: add on share, drop on unshare.
			publicFiles = updated.public
				? publicFiles.some((x) => x.id === updated.id)
					? publicFiles.map((x) => (x.id === updated.id ? updated : x))
					: [updated, ...publicFiles]
				: publicFiles.filter((x) => x.id !== updated.id);
			if (propsFile?.id === f.id) propsFile = updated;
		} catch (e) {
			fail(e, 'Could not update sharing');
		}
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

		// Escape leaves the search results view back to the plain section.
		if (e.key === 'Escape' && query) { goto(`/files/${sectionParam}`); return; }

		if (e.key === 'j' || e.key === 'ArrowDown' || e.key === 'k' || e.key === 'ArrowUp') {
			const ids = displayFiles.map((f) => f.id);
			if (!ids.length) return;
			const idx = ids.indexOf(activeId ?? '');
			const next = (e.key === 'j' || e.key === 'ArrowDown')
				? (idx < 0 ? 0 : Math.min(ids.length - 1, idx + 1))
				: (idx < 0 ? 0 : Math.max(0, idx - 1));
			activeId = ids[next];
			e.preventDefault();
		}
		if ((e.key === 'Enter' || e.key === 'o') && activeId && sectionParam !== 'trash') {
			const f = displayFiles.find((x) => x.id === activeId);
			if (f) handleView(f);
			e.preventDefault();
		}
		if ((e.key === 'Delete' || e.key === 'Backspace') && activeId && sectionParam !== 'trash') {
			const f = displayFiles.find((x) => x.id === activeId);
			if (f) deleteTarget = f;
			e.preventDefault();
		}
		if (e.key === 'r' && activeId) {
			const f = displayFiles.find((x) => x.id === activeId);
			if (f) renameTarget = f;
			e.preventDefault();
		}
		if ((e.key === 'i' || e.key === ' ') && activeId) {
			const f = displayFiles.find((x) => x.id === activeId);
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
	crumbs={['fileserve.rs', query ? `Search: “${query}”` : TITLES[sectionParam] ?? 'Files']}
	searchValue={query ?? ''}
	onPick={handleView}
	onSubmit={handleSearchSubmit}
	onUpload={() => (showUpload = true)}
	dark={prefs.dark}
	onToggleDark={() => (prefs.dark = !prefs.dark)}
	user={authStore.user!}
	onLogout={() => { authStore.logout(); goto('/login'); }}
	onSettings={() => goto('/settings')}
	bind:searchRef={searchEl}
/>

<FileBrowser
	files={displayFiles}
	view={prefs.view}
	section={sectionParam}
	search={query ?? ''}
	{activeId}
	onActiveId={(id) => (activeId = id)}
	onViewChange={(v) => (prefs.view = v)}
	onView={handleView}
	onDownload={handleDownload}
	onRename={(f) => (renameTarget = f)}
	onRestore={handleRestore}
	onDelete={(f) => (deleteTarget = f)}
	onProperties={(f) => (propsFile = f)}
	onUpload={() => (showUpload = true)}
/>

{#if showUpload}
	<UploadModal onClose={() => (showUpload = false)} onComplete={refresh} />
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
