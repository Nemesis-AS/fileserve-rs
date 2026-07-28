<script lang="ts">
	import Icon from './Icon.svelte';
	import { uploadsStore } from '$lib/stores/uploads.svelte';
	import { Modal } from './ui/modal/index.js';
	import { Button } from './ui/button/index.js';
	import { Dropzone } from './ui/dropzone/index.js';

	let { onClose }: { onClose: () => void } = $props();

	let drag = $state(false);
	let inputEl: HTMLInputElement | undefined = $state();

	/**
	 * The dialog only starts transfers — progress lives in the dock, so picking
	 * files hands them off and gets out of the way.
	 */
	function accept(files: FileList | null) {
		if (!files?.length) return;
		uploadsStore.add(files);
		onClose();
	}

	function onDrop(e: DragEvent) {
		e.preventDefault();
		drag = false;
		accept(e.dataTransfer?.files ?? null);
	}

	function onPick(e: Event) {
		const input = e.target as HTMLInputElement;
		accept(input.files);
		input.value = '';
	}
</script>

<Modal title="Upload files" {onClose}>
	<Dropzone
		active={drag}
		hint="Select one or more files"
		ondragover={(e: DragEvent) => {
			e.preventDefault();
			drag = true;
		}}
		ondragleave={() => (drag = false)}
		ondrop={onDrop}
		onclick={() => inputEl?.click()}
	>
		<Icon name="Upload" class="mx-auto" size={28} />
		<div class="text-[13px] font-medium text-ink">
			Drop files here or <span class="text-accent-ink">browse</span>
		</div>
		<input bind:this={inputEl} type="file" multiple class="hidden" onchange={onPick} />
	</Dropzone>

	{#snippet footer()}
		<Button variant="ghost" onclick={onClose}>Cancel</Button>
	{/snippet}
</Modal>
