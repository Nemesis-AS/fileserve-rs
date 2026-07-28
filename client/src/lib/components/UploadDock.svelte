<script lang="ts">
	import { fly, slide } from 'svelte/transition';
	import Icon from './Icon.svelte';
	import { fmtSize } from '$lib/utils/file';
	import { uploadsStore } from '$lib/stores/uploads.svelte';
	import { IconButton } from './ui/icon-button/index.js';
	import { Meter } from './ui/meter/index.js';

	const items = $derived(uploadsStore.items);
	const active = $derived(uploadsStore.activeCount);
	const done = $derived(uploadsStore.doneCount);
	const failed = $derived(uploadsStore.errorCount);

	const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? '' : 's'}`;

	const title = $derived.by(() => {
		if (active > 0) return `Uploading ${plural(active, 'item')}`;
		if (done === 0 && failed > 0) return `${plural(failed, 'upload')} failed`;
		return failed > 0
			? `${plural(done, 'upload')} complete · ${failed} failed`
			: `${plural(done, 'upload')} complete`;
	});
</script>

{#if uploadsStore.open && items.length > 0}
	<div
		class="fixed right-4 bottom-4 z-90 flex w-[min(360px,calc(100vw-2rem))] flex-col overflow-hidden rounded-xl border border-edge bg-surface shadow-lift"
		transition:fly={{ y: 16, duration: 200 }}
	>
		<div class="flex shrink-0 items-center gap-1 border-b border-edge bg-elevated py-2 pr-2 pl-3.5">
			<span class="min-w-0 flex-1 truncate text-[13px] font-medium text-ink">{title}</span>

			<IconButton
				size="sm"
				onclick={() => uploadsStore.toggleCollapsed()}
				aria-label={uploadsStore.collapsed ? 'Expand uploads' : 'Collapse uploads'}
				aria-expanded={!uploadsStore.collapsed}
			>
				<Icon
					name="ChevronD"
					size={15}
					class="transition-transform duration-200 {uploadsStore.collapsed ? '' : 'rotate-180'}"
				/>
			</IconButton>

			<IconButton size="sm" onclick={() => uploadsStore.dismiss()} aria-label="Close uploads">
				<Icon name="Close" size={13} />
			</IconButton>
		</div>

		{#if !uploadsStore.collapsed}
			<div transition:slide={{ duration: 180 }}>
				<div class="scroll-area max-h-[264px] overflow-y-auto py-1">
					{#each items as it (it.id)}
						<div
							class="grid grid-cols-[22px_1fr_auto_auto] items-center gap-2.5 px-3 py-1.5 text-[12.5px]"
						>
							<div
								class="grid size-5.5 shrink-0 place-items-center rounded-[5px] font-code text-[9px] font-bold text-white"
								style="background: {it.color};"
							>
								{it.ext.slice(0, 4).toUpperCase()}
							</div>

							<div class="flex min-w-0 flex-col gap-0.75">
								<b class="overflow-hidden font-[450] text-ellipsis whitespace-nowrap">{it.name}</b>
								{#if it.state === 'uploading'}
									<Meter
										value={it.progress}
										size="xs"
										radius="sharp"
										speed="fast"
										color="var(--accent)"
									/>
								{/if}
								<span
									class="text-[11px] text-ink-muted data-[state=done]:text-ok data-[state=error]:text-danger"
									data-state={it.state}
								>
									{#if it.state === 'uploading'}
										{fmtSize(Math.round((it.size * it.progress) / 100))} of {fmtSize(it.size)}
									{:else if it.state === 'done'}
										Uploaded · {fmtSize(it.size)}
									{:else}
										{it.error ?? 'Upload failed'}
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

							<IconButton
								size="xs"
								onclick={() => uploadsStore.remove(it.id)}
								aria-label={it.state === 'uploading' ? `Cancel ${it.name}` : `Dismiss ${it.name}`}
							>
								<Icon name="Close" size={12} />
							</IconButton>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</div>
{/if}
