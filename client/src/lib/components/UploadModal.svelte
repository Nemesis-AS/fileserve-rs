<script lang="ts">
	import Icon from './Icon.svelte';
	import { extOf, fileColor, fmtSize } from '$lib/utils/file';
	import { Modal } from './ui/modal/index.js';
	import { Button } from './ui/button/index.js';
	import { IconButton } from './ui/icon-button/index.js';
	import { Checkbox } from './ui/checkbox/index.js';
	import { Dropzone } from './ui/dropzone/index.js';
	import { Meter } from './ui/meter/index.js';

	let { onClose, onComplete }: { onClose: () => void; onComplete?: (files: UploadItem[]) => void } =
		$props();

	interface UploadItem {
		id: string;
		name: string;
		size: number;
		ext: string;
		color: string;
		progress: number;
		state: 'uploading' | 'done' | 'error';
	}

	let drag = $state(false);
	let queue = $state<UploadItem[]>([]);
	let makePublic = $state(false);
	let inputEl: HTMLInputElement | undefined = $state();
	const timers = new Map<string, ReturnType<typeof setInterval>>();

	function addFiles(fileList: FileList) {
		const items: UploadItem[] = Array.from(fileList).map((f, i) => ({
			id: 'q' + Date.now() + '-' + i,
			name: f.name,
			size: f.size,
			ext: extOf(f.name),
			color: fileColor(f.name),
			progress: 0,
			state: 'uploading' as const
		}));
		queue = [...queue, ...items];
		items.forEach((it) => simulateUpload(it.id));
	}

	function simulateUpload(id: string) {
		const speed = 8 + Math.random() * 14;
		timers.set(
			id,
			setInterval(() => {
				queue = queue.map((it) => {
					if (it.id !== id || it.state !== 'uploading') return it;
					const next = Math.min(100, it.progress + speed);
					if (next >= 100) {
						clearInterval(timers.get(id)!);
						timers.delete(id);
						return { ...it, progress: 100, state: Math.random() < 0.07 ? 'error' : 'done' };
					}
					return { ...it, progress: next };
				});
			}, 220)
		);
	}

	// Prefill with demo items
	$effect(() => {
		const demo = [
			{ name: 'thursday-notes.md', size: 14_220 },
			{ name: 'IMG_4128.jpeg', size: 2_982_001 }
		];
		const items: UploadItem[] = demo.map((f, i) => ({
			id: 'qd-' + i,
			name: f.name,
			size: f.size,
			ext: extOf(f.name),
			color: fileColor(f.name),
			progress: i === 0 ? 100 : 38,
			state: i === 0 ? 'done' : 'uploading'
		}));
		queue = items;
		simulateUpload('qd-1');
		return () => timers.forEach(clearInterval);
	});

	function onDrop(e: DragEvent) {
		e.preventDefault();
		drag = false;
		if (e.dataTransfer?.files) addFiles(e.dataTransfer.files);
	}

	function onPick(e: Event) {
		const input = e.target as HTMLInputElement;
		if (input.files) addFiles(input.files);
		input.value = '';
	}

	function removeItem(id: string) {
		const t = timers.get(id);
		if (t) {
			clearInterval(t);
			timers.delete(id);
		}
		queue = queue.filter((it) => it.id !== id);
	}

	const totalProgress = $derived(
		queue.length ? queue.reduce((s, q) => s + q.progress, 0) / queue.length : 0
	);
	const allDone = $derived(queue.length > 0 && queue.every((q) => q.state !== 'uploading'));
	const doneCount = $derived(queue.filter((q) => q.state === 'done').length);
</script>

<Modal title="Upload files" size="wide" {onClose}>
	<Dropzone
		active={drag}
		hint="Single files up to 5 GB · multiple selections OK"
		ondragover={(e: DragEvent) => {
			e.preventDefault();
			drag = true;
		}}
		ondragleave={() => (drag = false)}
		ondrop={onDrop}
		onclick={() => inputEl?.click()}
	>
		<!--
		  No mx-auto: preflight sets svg{display:block}, so the dropzone's text-align:center
		  never centred this icon — it sat at the left edge. Centring it would be a redesign.
		-->
		<Icon name="Upload" size={28} />
		<div class="text-[13px] font-medium text-ink">
			Drop files here or <span class="text-accent-ink">browse</span>
		</div>
		<input bind:this={inputEl} type="file" multiple class="hidden" onchange={onPick} />
	</Dropzone>

	<Checkbox bind:checked={makePublic} class="mt-[14px]">
		Make these files public (shareable link)
	</Checkbox>

	{#if queue.length > 0}
		<div class="mt-[18px] mb-1 flex items-baseline justify-between text-[12px] text-ink-muted">
			<span class="font-medium text-ink">
				Queue · {queue.length}
				{queue.length === 1 ? 'file' : 'files'}
			</span>
			<span>{allDone ? `${doneCount} uploaded` : `${Math.round(totalProgress)}%`}</span>
		</div>

		<!-- .upload-list -->
		<div class="flex flex-col gap-2">
			{#each queue as it (it.id)}
				<!-- .upload-row: icon | name+progress | pct | remove -->
				<div
					class="grid grid-cols-[22px_1fr_auto_auto] items-center gap-2.5 px-1 py-2 text-[12.5px]"
				>
					<div
						class="grid size-[22px] shrink-0 place-items-center rounded-[5px] font-code text-[9px] font-bold text-white"
						style="background: {it.color};"
					>
						{it.ext.slice(0, 4).toUpperCase()}
					</div>

					<div class="flex min-w-0 flex-col gap-[3px]">
						<b class="overflow-hidden font-[450] text-ellipsis whitespace-nowrap">{it.name}</b>
						<Meter
							value={it.progress}
							size="xs"
							radius="sharp"
							speed="fast"
							color={it.state === 'error' ? 'var(--danger)' : 'var(--accent)'}
						/>
						<span
							class="text-[11px] text-ink-muted data-[state=done]:text-ok data-[state=error]:text-danger"
							data-state={it.state}
						>
							{#if it.state === 'uploading'}
								{fmtSize(Math.round((it.size * it.progress) / 100))} of {fmtSize(it.size)}
							{:else if it.state === 'done'}
								Uploaded · {fmtSize(it.size)}
							{:else}
								Upload failed — retry?
							{/if}
						</span>
					</div>

					<div class="min-w-9 text-right text-[11.5px] text-ink-muted tabular-nums">
						{#if it.state === 'done'}
							<Icon name="Check" size={14} class="text-ok" />
						{:else if it.state === 'error'}
							—
						{:else}
							{Math.round(it.progress)}%
						{/if}
					</div>

					<IconButton size="xs" onclick={() => removeItem(it.id)} aria-label="Remove">
						<Icon name="Close" size={12} />
					</IconButton>
				</div>
			{/each}
		</div>
	{/if}

	{#snippet footer()}
		<Button variant="ghost" onclick={onClose}>
			{allDone ? 'Close' : 'Cancel'}
		</Button>
		{#if allDone}
			<Button
				onclick={() => {
					onComplete?.(queue);
					onClose();
				}}>Done</Button
			>
		{/if}
	{/snippet}
</Modal>
