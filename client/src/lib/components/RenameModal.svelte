<script lang="ts">
	import type { FilerFile } from '$lib/types';
	import { Modal } from './ui/modal/index.js';
	import { Button } from './ui/button/index.js';
	import { Field } from './ui/field/index.js';
	import { Input } from './ui/input/index.js';

	let {
		file,
		onClose,
		onRename
	}: {
		file: FilerFile;
		onClose: () => void;
		onRename: (newName: string) => void;
	} = $props();

	let name = $state(file.name);
	let inputEl: HTMLInputElement | undefined = $state();

	$effect(() => {
		if (!inputEl) return;
		inputEl.focus();
		const dot = file.name.lastIndexOf('.');
		if (dot > 0) inputEl.setSelectionRange(0, dot);
		else inputEl.select();
	});

	function submit(e: SubmitEvent) {
		e.preventDefault();
		const trimmed = name.trim();
		if (trimmed) onRename(trimmed);
	}
</script>

<!-- onsubmit makes Modal render a <form> root, preserving Enter-to-submit. -->
<Modal title="Rename file" {onClose} onsubmit={submit}>
	<Field label="New name" for="rename-input" class="mb-0">
		<Input id="rename-input" bind:ref={inputEl} size="lg" bind:value={name} />
	</Field>

	{#snippet footer()}
		<Button variant="ghost" onclick={onClose}>Cancel</Button>
		<Button type="submit">Rename</Button>
	{/snippet}
</Modal>
