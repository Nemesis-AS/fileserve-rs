<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import FileIcon from '$lib/components/FileIcon.svelte';
	import AudioPlayer from '$lib/components/AudioPlayer.svelte';
	import VideoPlayer from '$lib/components/VideoPlayer.svelte';
	import PropertiesModal from '$lib/components/PropertiesModal.svelte';
	import DeleteConfirm from '$lib/components/DeleteConfirm.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import {
		getFiles,
		getPublicFiles,
		toggleShare,
		downloadFile,
		trashFile,
		restoreFile,
		deleteFile,
		previewUrl
	} from '$lib/services/files';
	import type { FilerFile } from '$lib/types';
	import { fmtSize, fmtDate, isTextPreviewable } from '$lib/utils/file';
	import { clickOutside } from '$lib/actions/clickOutside';
	import { Button } from '$lib/components/ui/button/index.js';
	import { IconButton } from '$lib/components/ui/icon-button/index.js';
	import { BackButton } from '$lib/components/ui/back-button/index.js';
	import { Menu, MenuItem, MenuSeparator } from '$lib/components/ui/menu/index.js';

	const fileId = $derived($page.params.id);
	const section = $derived($page.params.section);

	let file = $state<FilerFile | null>(null);
	let propsFile = $state<FilerFile | null>(null);
	let menuOpen = $state(false);
	let showDelete = $state(false);

	// Text preview: rendering a huge file into a <pre> would freeze the tab, so
	// we cap what we'll fetch and offer download instead past the limit.
	const MAX_TEXT_PREVIEW_BYTES = 2 * 1024 * 1024;
	type TextState =
		| { status: 'loading' }
		| { status: 'ready'; body: string }
		| { status: 'too-large' }
		| { status: 'error' };
	let textState = $state<TextState | null>(null);
	let imgFailed = $state(false);

	// Load the raw bytes for text-like files once the record resolves. Keyed on
	// file.id so it re-runs if the viewer is reused for a different file.
	$effect(() => {
		const f = file;
		imgFailed = false;
		if (!f || f.category === 'img' || !isTextPreviewable(f.name)) {
			textState = null;
			return;
		}
		if (f.size > MAX_TEXT_PREVIEW_BYTES) {
			textState = { status: 'too-large' };
			return;
		}

		let cancelled = false;
		textState = { status: 'loading' };
		fetch(previewUrl(f.id), { credentials: 'include' })
			.then((res) => {
				if (!res.ok) throw new Error(`HTTP ${res.status}`);
				return res.text();
			})
			.then((body) => {
				if (!cancelled) textState = { status: 'ready', body };
			})
			.catch(() => {
				if (!cancelled) textState = { status: 'error' };
			});

		return () => {
			cancelled = true;
		};
	});

	onMount(async () => {
		try {
			// A public file may belong to another user, so it won't be in the
			// caller's own list — resolve it from the cross-owner listing instead.
			const files = section === 'public' ? await getPublicFiles() : await getFiles();
			file = files.find((f) => f.id === fileId) ?? null;
		} catch (e) {
			toastStore.show(e instanceof Error ? e.message : 'Could not load file');
		}
	});

	function handleDownload() {
		if (file) downloadFile(file);
	}

	async function handleToggleShare(f: FilerFile, makePublic: boolean) {
		try {
			const updated = await toggleShare(f, makePublic);
			file = updated;
			if (propsFile) propsFile = updated;
		} catch (e) {
			toastStore.show(e instanceof Error ? e.message : 'Could not update sharing');
		}
	}

	/** Trash (or permanently delete, in the Trash section) then return to the list — the file leaves this view either way. */
	async function handleDelete() {
		if (!file) return;
		const f = file;
		showDelete = false;
		try {
			if (section === 'trash') {
				await deleteFile(f.id);
				toastStore.show(`Deleted "${f.name}" permanently`);
			} else {
				await trashFile(f.id);
				// Undo restores server-side too, matching the list view's behaviour.
				toastStore.show(`Moved "${f.name}" to Trash`, async () => {
					try {
						await restoreFile(f.id);
					} catch (e) {
						toastStore.show(e instanceof Error ? e.message : 'Could not restore file');
					}
				});
			}
			goto(`/files/${section}`);
		} catch (e) {
			toastStore.show(e instanceof Error ? e.message : 'Could not delete file');
		}
	}

	async function handleRestore() {
		if (!file) return;
		const f = file;
		menuOpen = false;
		try {
			await restoreFile(f.id);
			toastStore.show(`Restored "${f.name}"`);
			goto(`/files/${section}`);
		} catch (e) {
			toastStore.show(e instanceof Error ? e.message : 'Could not restore file');
		}
	}

	// .viewer__fallback — shared by the img/vid/aud/unknown branches below
	const fallback = 'flex flex-col items-center gap-3 text-center text-ink-muted';
</script>

<TopBar
	crumbs={['fileserve.rs', 'Files']}
	dark={prefs.dark}
	onToggleDark={() => (prefs.dark = !prefs.dark)}
	user={authStore.user!}
	onLogout={() => {
		authStore.logout();
		goto('/login');
	}}
	onSettings={() => goto('/settings')}
/>

{#if !file}
	<div class="grid flex-1 place-items-center text-[13.5px] text-ink-muted">Loading…</div>
{:else}
	<div class="flex min-h-0 flex-1 flex-col bg-sunken">
		<!-- .viewer__bar -->
		<div class="flex h-12 shrink-0 items-center gap-2.5 border-b border-edge bg-surface px-4">
			<BackButton onclick={() => goto(`/files/${section}`)}>Back</BackButton>

			<div class="flex min-w-0 items-center gap-2 text-[13.5px]">
				<FileIcon {file} />
				<b class="overflow-hidden font-medium text-ellipsis whitespace-nowrap">{file.name}</b>
				<span class="text-[12px] whitespace-nowrap text-ink-faint">
					· {fmtSize(file.size)} · {fmtDate(file.modified)}
				</span>
			</div>

			<div class="ml-auto flex shrink-0 gap-1.5">
				<Button variant="ghost" onclick={() => (propsFile = file)}>
					<Icon name="Info" size={14} />
					Properties
				</Button>
				{#if section !== 'trash'}
					<Button onclick={handleDownload}>
						<Icon name="Download" size={14} />
						Download
					</Button>
				{/if}

				<div
					class="relative"
					use:clickOutside={{ enabled: menuOpen, onOutside: () => (menuOpen = false) }}
				>
					<IconButton title="More options" onclick={() => (menuOpen = !menuOpen)}>
						<Icon name="More" size={16} />
					</IconButton>

					{#if menuOpen}
						<Menu class="top-[calc(100%+6px)] right-0">
							{#if section === 'trash'}
								<MenuItem onclick={handleRestore}>
									<Icon name="Refresh" size={14} />Restore
								</MenuItem>
								<MenuSeparator />
								<MenuItem
									danger
									onclick={() => {
										menuOpen = false;
										showDelete = true;
									}}
								>
									<Icon name="Trash" size={14} />Delete permanently
								</MenuItem>
							{:else}
								<MenuItem
									danger
									onclick={() => {
										menuOpen = false;
										showDelete = true;
									}}
								>
									<Icon name="Trash" size={14} />Move to Trash
								</MenuItem>
							{/if}
						</Menu>
					{/if}
				</div>
			</div>
		</div>

		<!-- .viewer__stage -->
		<div class="scroll-area grid flex-1 place-items-center overflow-auto p-8">
			{#if file.category === 'img' && !imgFailed}
				<img
					class="max-h-full max-w-full rounded-lg shadow-pop"
					src={file.thumb ? file.thumb.replace(/w=400/, 'w=1600') : previewUrl(file.id)}
					alt={file.name}
					onerror={() => (imgFailed = true)}
				/>
			{:else if file.category === 'img'}
				<div class={fallback}>
					<FileIcon {file} large />
					<div class="text-[14px]">Preview unavailable</div>
					<div class="text-[12.5px]">This image couldn't be shown. Download it to view.</div>
				</div>
			{:else if isTextPreviewable(file.name)}
				<!-- .viewer__text -->
				{#if textState?.status === 'ready'}
					<pre
						class="scroll-area max-h-full w-[min(820px,100%)] overflow-auto rounded-[10px] border border-edge bg-surface px-9 py-7 font-code text-[13px] leading-[1.7] whitespace-pre-wrap text-ink shadow-xs">{textState.body}</pre>
				{:else if textState?.status === 'loading'}
					<div class={fallback}>
						<div class="text-[13.5px]">Loading preview…</div>
					</div>
				{:else if textState?.status === 'too-large'}
					<div class={fallback}>
						<FileIcon {file} large />
						<div class="text-[14px]">File too large to preview</div>
						<div class="text-[12.5px]">This file is {fmtSize(file.size)}. Download it to view.</div>
						{#if section !== 'trash'}
							<Button onclick={handleDownload}>
								<Icon name="Download" size={14} />Download
							</Button>
						{/if}
					</div>
				{:else}
					<div class={fallback}>
						<FileIcon {file} large />
						<div class="text-[14px]">Couldn't load preview</div>
						<div class="text-[12.5px]">Download it to view in another app.</div>
						{#if section !== 'trash'}
							<Button onclick={handleDownload}>
								<Icon name="Download" size={14} />Download
							</Button>
						{/if}
					</div>
				{/if}
			{:else if file.category === 'pdf'}
				<!-- .viewer__doc — browser's native PDF viewer -->
				<iframe
					class="h-full w-full max-w-[1100px] rounded-[10px] border border-edge bg-surface shadow-xs"
					src={previewUrl(file.id)}
					title={file.name}
				></iframe>
			{:else if file.category === 'vid'}
				<VideoPlayer src={previewUrl(file.id)} name={file.name} />
			{:else if file.category === 'aud'}
				<AudioPlayer src={previewUrl(file.id)} name={file.name} />
			{:else}
				<div class={fallback}>
					<FileIcon {file} large />
					<div class="text-[14px]">This file type can't be previewed</div>
					<div class="text-[12.5px]">Download it to view in another app.</div>
					{#if section !== 'trash'}
						<Button onclick={handleDownload}>
							<Icon name="Download" size={14} />Download
						</Button>
					{/if}
				</div>
			{/if}
		</div>
	</div>

	{#if propsFile}
		<PropertiesModal
			file={propsFile}
			onClose={() => (propsFile = null)}
			onToggleShare={handleToggleShare}
		/>
	{/if}

	{#if showDelete}
		<DeleteConfirm
			file={file!}
			inTrash={section === 'trash'}
			onClose={() => (showDelete = false)}
			onConfirm={handleDelete}
		/>
	{/if}
{/if}
