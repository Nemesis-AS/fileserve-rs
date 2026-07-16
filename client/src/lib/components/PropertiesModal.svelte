<script lang="ts">
	import Icon from './Icon.svelte';
	import FileIcon from './FileIcon.svelte';
	import type { FilerFile } from '$lib/types';
	import { fmtSize, fmtDateLong } from '$lib/utils/file';
	import { Modal } from './ui/modal/index.js';
	import { Button } from './ui/button/index.js';
	import { IconButton } from './ui/icon-button/index.js';

	let {
		file,
		onClose,
		onToggleShare
	}: {
		file: FilerFile;
		onClose: () => void;
		onToggleShare: (f: FilerFile, makePublic: boolean) => void;
	} = $props();

	let copied = $state(false);
	const shareUrl = $derived(
		`https://files.home.lan/s/${file.id}-${file.name.replace(/[^a-z0-9.]/gi, '-')}`
	);

	function copy() {
		navigator.clipboard?.writeText(shareUrl).catch(() => {});
		copied = true;
		setTimeout(() => (copied = false), 1500);
	}
</script>

<Modal title="Properties" {onClose}>
	<!-- .props__preview -->
	<div
		class="mb-[14px] grid aspect-video place-items-center overflow-hidden rounded-lg border border-edge bg-sunken bg-cover bg-center"
		style={file.thumb ? `background-image: url(${file.thumb})` : undefined}
	>
		{#if !file.thumb}
			<FileIcon {file} large />
		{/if}
	</div>

	<!-- .props: a 100px label column against free-form values -->
	<dl class="grid grid-cols-[100px_1fr] gap-x-[18px] gap-y-3 py-1 pb-2 text-[13px]">
		<dt class="font-normal text-ink-muted">Name</dt>
		<dd class="m-0 break-words text-ink tabular-nums">{file.name}</dd>

		<dt class="font-normal text-ink-muted">Type</dt>
		<dd class="m-0 break-words text-ink tabular-nums">{file.ext.toUpperCase()} · {file.category}</dd>

		<dt class="font-normal text-ink-muted">Size</dt>
		<dd class="m-0 break-words text-ink tabular-nums">
			{fmtSize(file.size)}
			<span class="text-ink-faint">({file.size.toLocaleString()} bytes)</span>
		</dd>

		<dt class="font-normal text-ink-muted">Modified</dt>
		<dd class="m-0 break-words text-ink tabular-nums">{fmtDateLong(file.modified)}</dd>

		<dt class="font-normal text-ink-muted">Owner</dt>
		<dd class="m-0 break-words text-ink tabular-nums">{file.owner === 'me' ? 'You' : file.owner}</dd>

		<dt class="font-normal text-ink-muted">Path</dt>
		<dd class="m-0 break-words text-ink tabular-nums">
			<code class="font-code text-[12px]">/{file.name}</code>
		</dd>

		<dt class="font-normal text-ink-muted">Checksum</dt>
		<dd class="m-0 break-words text-ink tabular-nums">
			<code class="font-code text-[12px]">sha256:{file.id.padEnd(8, '0')}…{file.ext}</code>
		</dd>

		<dt class="font-normal text-ink-muted">Visibility</dt>
		<dd class="m-0 break-words text-ink tabular-nums">
			<label class="inline-flex cursor-pointer items-center gap-2 text-[13px]">
				<input
					type="checkbox"
					class="accent-accent"
					checked={file.public}
					onchange={(e) => onToggleShare(file, (e.target as HTMLInputElement).checked)}
				/>
				Public — accessible via link
			</label>
		</dd>
	</dl>

	{#if file.public}
		<!-- .share-link -->
		<div
			class="mt-[14px] flex items-center gap-2 overflow-hidden rounded-[7px] border border-edge bg-sunken px-3 py-2.5 font-code text-[11.5px] text-ink-muted"
		>
			<Icon name="Link" size={14} class="shrink-0" />
			<code class="flex-1 overflow-hidden text-ellipsis">{shareUrl}</code>
			<IconButton
				variant="row"
				size="sm"
				title={copied ? 'Copied!' : 'Copy link'}
				onclick={copy}
			>
				{#if copied}
					<Icon name="Check" size={14} class="text-ok" />
				{:else}
					<Icon name="Copy" size={14} />
				{/if}
			</IconButton>
		</div>
	{/if}

	{#snippet footer()}
		<Button variant="ghost" onclick={onClose}>Close</Button>
	{/snippet}
</Modal>
