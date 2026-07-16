<script lang="ts">
	import type { FilerFile } from '$lib/types';
	import { tv } from './ui/tv.js';

	/**
	 * Port of `.file-name__icon`.
	 *
	 * Only two sizes exist. layout.css also carried `.viewer__fallback .file-name__icon`
	 * (72px) and `.upload-row .file-name__icon` (9px), but the old component set width /
	 * height / font-size as *inline* styles, which outrank any selector — so both rules
	 * were dead and those icons really render at 56px/12px and 8px. Verified in-browser
	 * before porting. Don't "restore" them.
	 *
	 * Likewise `[data-thumb='1']` declared `border-radius: 4px`, also dead against the
	 * inline radius; only its `overflow: hidden` ever applied.
	 */
	const fileIcon = tv({
		base: 'grid shrink-0 place-items-center bg-cover bg-center font-code font-bold tracking-normal text-white',
		variants: {
			size: {
				sm: 'size-[22px] rounded-[5px] text-[8px]',
				lg: 'size-14 rounded-[10px] text-[12px]'
			},
			thumb: { true: 'overflow-hidden', false: '' }
		},
		defaultVariants: { size: 'sm', thumb: false }
	});

	let {
		file,
		large = false,
		class: klass
	}: { file: FilerFile; large?: boolean; class?: string } = $props();

	const size = $derived(large ? 'lg' : 'sm');
</script>

{#if file.thumb}
	<div
		class={fileIcon({ size, thumb: true, class: klass })}
		data-thumb="1"
		style="background-image: url({file.thumb});"
	></div>
{:else}
	<div class={fileIcon({ size, class: klass })} style="background: {file.color};">
		{file.ext.slice(0, 4).toUpperCase()}
	</div>
{/if}
