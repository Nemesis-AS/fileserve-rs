<script lang="ts">
	import Icon from './Icon.svelte';
	import type { FileSection } from '$lib/types';
	import { EmptyState as UiEmptyState } from './ui/empty-state/index.js';
	import { Button } from './ui/button/index.js';

	let {
		section,
		search = '',
		onUpload
	}: {
		section: FileSection;
		search?: string;
		onUpload?: (() => void) | null;
	} = $props();

	const title = $derived(() => {
		if (search) return 'No matches';
		if (section === 'public') return 'Nothing shared yet';
		if (section === 'trash') return 'Trash is empty';
		return 'No files yet';
	});

	const body = $derived(() => {
		if (search) return `Nothing matched "${search}". Try a different search.`;
		if (section === 'public') return 'Files you and other users mark as public will appear here.';
		if (section === 'trash')
			return 'Deleted files appear here for 30 days before being permanently removed.';
		return 'Upload your first file to get started.';
	});
</script>

<UiEmptyState title={title()} body={body()}>
	{#snippet art()}
		{#if section === 'trash'}
			<Icon name="TrashBin" size={28} />
		{:else if section === 'public'}
			<Icon name="Public" size={28} />
		{:else}
			<Icon name="Folder" size={28} />
		{/if}
	{/snippet}

	{#if onUpload && !search}
		<Button onclick={onUpload}>
			<Icon name="Upload" size={14} />
			Upload files
		</Button>
	{/if}
</UiEmptyState>
