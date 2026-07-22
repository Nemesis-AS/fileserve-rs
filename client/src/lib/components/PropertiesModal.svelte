<script lang="ts">
	import Icon from './Icon.svelte';
	import FileIcon from './FileIcon.svelte';
	import { page } from '$app/stores';
	import type { FilerFile } from '$lib/types';
	import { fmtSize, fmtDateLong } from '$lib/utils/file';
	import { createShareLink, downloadUrl } from '$lib/services/files';
	import { authStore } from '$lib/stores/auth.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { Modal } from './ui/modal/index.js';
	import { Button } from './ui/button/index.js';
	import { IconButton } from './ui/icon-button/index.js';
	import { Select } from './ui/input/index.js';

	let {
		file,
		onClose,
		onToggleShare
	}: {
		file: FilerFile;
		onClose: () => void;
		onToggleShare: (f: FilerFile, makePublic: boolean) => void;
	} = $props();

	const canShare = $derived(file.owner === authStore.user?.username);

	let copied = $state(false);
	const shareUrl = $derived(`${$page.url.origin}/files/public/${file.id}`);

	function copy() {
		navigator.clipboard?.writeText(shareUrl).catch(() => {});
		copied = true;
		setTimeout(() => (copied = false), 1500);
	}

	const EXPIRY_OPTIONS = [
		{ label: '1 hour', minutes: 60 },
		{ label: '24 hours', minutes: 1440 },
		{ label: '7 days', minutes: 10080 }
	];
	let expiryMinutes = $state('1440');
	let creatingLink = $state(false);
	let tokenLink = $state<{ url: string; expiresAt: string } | null>(null);
	let tokenCopied = $state(false);

	async function createLink() {
		creatingLink = true;
		try {
			const { token, expiresAt } = await createShareLink(file.id, Number(expiryMinutes));
			tokenLink = {
				url: `${$page.url.origin}${downloadUrl(file.id)}?token=${encodeURIComponent(token)}`,
				expiresAt
			};
		} catch (e) {
			toastStore.show(e instanceof Error ? e.message : 'Could not create share link');
		} finally {
			creatingLink = false;
		}
	}

	function copyToken() {
		if (!tokenLink) return;
		navigator.clipboard?.writeText(tokenLink.url).catch(() => {});
		tokenCopied = true;
		setTimeout(() => (tokenCopied = false), 1500);
	}
</script>

<Modal title="Properties" {onClose}>
	<div
		class="mb-[14px] grid aspect-video place-items-center overflow-hidden rounded-lg border border-edge bg-sunken bg-cover bg-center"
		style={file.thumb ? `background-image: url(${file.thumb})` : undefined}
	>
		{#if !file.thumb}
			<FileIcon {file} large />
		{/if}
	</div>

	<dl class="grid grid-cols-[100px_1fr] gap-x-[18px] gap-y-3 py-1 pb-2 text-[13px]">
		<dt class="font-normal text-ink-muted">Name</dt>
		<dd class="m-0 break-words text-ink tabular-nums">{file.name}</dd>

		<dt class="font-normal text-ink-muted">Type</dt>
		<dd class="m-0 break-words text-ink tabular-nums">
			{file.ext.toUpperCase()} · {file.category}
		</dd>

		<dt class="font-normal text-ink-muted">Size</dt>
		<dd class="m-0 break-words text-ink tabular-nums">
			{fmtSize(file.size)}
			<span class="text-ink-faint">({file.size.toLocaleString()} bytes)</span>
		</dd>

		<dt class="font-normal text-ink-muted">Modified</dt>
		<dd class="m-0 break-words text-ink tabular-nums">{fmtDateLong(file.modified)}</dd>

		<dt class="font-normal text-ink-muted">Owner</dt>
		<dd class="m-0 break-words text-ink tabular-nums">
			{file.owner === 'me' ? 'You' : file.owner}
		</dd>

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
			{#if canShare}
				<label class="inline-flex cursor-pointer items-center gap-2 text-[13px]">
					<input
						type="checkbox"
						class="accent-accent"
						checked={file.public}
						onchange={(e) => onToggleShare(file, (e.target as HTMLInputElement).checked)}
					/>
					Public — visible to all signed-in users
				</label>
			{:else}
				<span class="text-[13px]">{file.public ? 'Public' : 'Private'}</span>
			{/if}
		</dd>
	</dl>

	{#if file.public && canShare}
		<div class="mt-[14px]">
			<div class="mb-2 text-[12px] font-medium text-ink">Public link</div>
			<div
				class="flex items-center gap-2 overflow-hidden rounded-[7px] border border-edge bg-sunken px-3 py-2.5 font-code text-[11.5px] text-ink-muted"
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
		</div>
	{/if}

	{#if canShare}
		<div class="mt-[18px] border-t border-edge pt-[14px]">
			<div class="mb-2 text-[12px] font-medium text-ink">Share link</div>
			<p class="mb-2.5 text-[12px] text-ink-muted">
				Anyone with the link can download this file until it expires — no sign-in needed.
			</p>
			<div class="flex items-center gap-2">
				<Select variant="filter" bind:value={expiryMinutes}>
					{#each EXPIRY_OPTIONS as opt (opt.minutes)}
						<option value={String(opt.minutes)}>{opt.label}</option>
					{/each}
				</Select>
				<Button onclick={createLink} disabled={creatingLink}>
					{creatingLink ? 'Creating…' : tokenLink ? 'New link' : 'Create link'}
				</Button>
			</div>

			{#if tokenLink}
				<div
					class="mt-2.5 flex items-center gap-2 overflow-hidden rounded-[7px] border border-edge bg-sunken px-3 py-2.5 font-code text-[11.5px] text-ink-muted"
				>
					<Icon name="Link" size={14} class="shrink-0" />
					<code class="flex-1 overflow-hidden text-ellipsis">{tokenLink.url}</code>
					<IconButton
						variant="row"
						size="sm"
						title={tokenCopied ? 'Copied!' : 'Copy link'}
						onclick={copyToken}
					>
						{#if tokenCopied}
							<Icon name="Check" size={14} class="text-ok" />
						{:else}
							<Icon name="Copy" size={14} />
						{/if}
					</IconButton>
				</div>
				<p class="mt-1.5 text-[11.5px] text-ink-faint">Expires {fmtDateLong(tokenLink.expiresAt)}</p>
			{/if}
		</div>
	{/if}

	{#snippet footer()}
		<Button variant="ghost" onclick={onClose}>Close</Button>
	{/snippet}
</Modal>
