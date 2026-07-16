<script lang="ts">
	import type { FilerFile } from '$lib/types';
	import { Modal } from './ui/modal/index.js';
	import { Button } from './ui/button/index.js';

	let {
		file,
		inTrash,
		onClose,
		onConfirm
	}: {
		file: FilerFile;
		inTrash: boolean;
		onClose: () => void;
		onConfirm: () => void;
	} = $props();
</script>

<Modal title={inTrash ? 'Permanently delete?' : 'Move to Trash?'} {onClose}>
	<p class="m-0 text-[13.5px]">
		<b>{file.name}</b><br />
		<span class="text-ink-muted">
			{inTrash
				? 'This file will be permanently deleted. This cannot be undone.'
				: 'You can restore this file from Trash within 30 days.'}
		</span>
	</p>

	{#snippet footer()}
		<Button variant="ghost" onclick={onClose}>Cancel</Button>
		<Button variant={inTrash ? 'danger' : 'solid'} onclick={onConfirm}>
			{inTrash ? 'Delete permanently' : 'Move to Trash'}
		</Button>
	{/snippet}
</Modal>
