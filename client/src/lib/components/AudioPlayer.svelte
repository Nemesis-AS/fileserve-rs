<script lang="ts">
	import PlayIcon from 'virtual:icons/heroicons/play-solid';
	import PauseIcon from 'virtual:icons/heroicons/pause-solid';
	import SpeakerWave from 'virtual:icons/heroicons/speaker-wave';
	import SpeakerXMark from 'virtual:icons/heroicons/speaker-x-mark';
	import MusicalNote from 'virtual:icons/heroicons/musical-note';
	import { formatTime as fmt, makeSlider } from '$lib/utils/player';

	let { src, name }: { src: string; name: string } = $props();

	let audio = $state<HTMLAudioElement>();
	// Bound to the <audio> element: currentTime/volume/muted are two-way (setting
	// them controls playback), paused/duration reflect the element's state.
	let paused = $state(true);
	let currentTime = $state(0);
	let duration = $state(0);
	let volume = $state(1);
	let muted = $state(false);
	let errored = $state(false);

	const progress = $derived(duration ? (currentTime / duration) * 100 : 0);
	const volPct = $derived((muted ? 0 : volume) * 100);

	function toggle() {
		if (!audio) return;
		if (audio.paused) audio.play().catch(() => (errored = true));
		else audio.pause();
	}

	const seek = makeSlider((r) => {
		if (duration) currentTime = r * duration;
	});
	const vol = makeSlider((r) => {
		volume = r;
		muted = r === 0;
	});

	function seekKeys(e: KeyboardEvent) {
		if (!duration) return;
		if (e.key === 'ArrowRight') currentTime = Math.min(duration, currentTime + 5);
		else if (e.key === 'ArrowLeft') currentTime = Math.max(0, currentTime - 5);
		else if (e.key === 'Home') currentTime = 0;
		else if (e.key === 'End') currentTime = duration;
		else return;
		e.preventDefault();
	}
</script>

<div
	class="w-[min(560px,100%)] rounded-[12px] border border-edge bg-surface p-5 shadow-xs"
>
	<audio
		bind:this={audio}
		{src}
		preload="metadata"
		bind:paused
		bind:currentTime
		bind:duration
		bind:volume
		bind:muted
		onerror={() => (errored = true)}
	></audio>

	<div class="mb-4 flex items-center gap-2.5 text-[13.5px]">
		<MusicalNote class="size-4 shrink-0 text-ink-muted" />
		<span class="overflow-hidden font-medium text-ellipsis whitespace-nowrap">{name}</span>
	</div>

	{#if errored}
		<div class="py-2 text-[12.5px] text-ink-muted">
			This audio couldn't be played. Try downloading it instead.
		</div>
	{:else}
		<div class="flex items-center gap-4">
			<button
				type="button"
				onclick={toggle}
				aria-label={paused ? 'Play' : 'Pause'}
				class="grid size-11 shrink-0 cursor-pointer place-items-center rounded-full bg-accent text-ink transition-opacity duration-100 hover:opacity-90 [&_svg]:size-4.5"
			>
				{#if paused}
					<PlayIcon />
				{:else}
					<PauseIcon />
				{/if}
			</button>

			<div class="min-w-0 flex-1">
				<!-- seek -->
				<div
					role="slider"
					tabindex="0"
					aria-label="Seek"
					aria-valuemin={0}
					aria-valuemax={Math.floor(duration) || 0}
					aria-valuenow={Math.floor(currentTime)}
					aria-valuetext={`${fmt(currentTime)} of ${fmt(duration)}`}
					onpointerdown={seek.down}
					onpointermove={seek.move}
					onpointerup={seek.up}
					onkeydown={seekKeys}
					class="group relative flex h-4 cursor-pointer touch-none items-center outline-none"
				>
					<div class="h-1.5 w-full overflow-hidden rounded-full bg-edge-strong">
						<div class="h-full rounded-full bg-accent" style="width: {progress}%"></div>
					</div>
					<div
						class="pointer-events-none absolute top-1/2 size-3 -translate-x-1/2 -translate-y-1/2 rounded-full bg-accent shadow-xs ring-2 ring-surface transition-transform group-hover:scale-110 group-focus-visible:scale-110"
						style="left: {progress}%"
					></div>
				</div>

				<div class="mt-1.5 flex justify-between text-[11.5px] tabular-nums text-ink-faint">
					<span>{fmt(currentTime)}</span>
					<span>{fmt(duration)}</span>
				</div>
			</div>

			<!-- volume -->
			<div class="flex shrink-0 items-center gap-2">
				<button
					type="button"
					onclick={() => (muted = !muted)}
					aria-label={muted ? 'Unmute' : 'Mute'}
					class="grid size-7 shrink-0 cursor-pointer place-items-center rounded-[6px] text-ink-muted transition-colors duration-100 hover:bg-row-hover hover:text-ink [&_svg]:size-4"
				>
					{#if muted || volume === 0}
						<SpeakerXMark />
					{:else}
						<SpeakerWave />
					{/if}
				</button>
				<div
					role="slider"
					tabindex="0"
					aria-label="Volume"
					aria-valuemin={0}
					aria-valuemax={100}
					aria-valuenow={Math.round(volPct)}
					onpointerdown={vol.down}
					onpointermove={vol.move}
					onpointerup={vol.up}
					onkeydown={(e) => {
						if (e.key === 'ArrowRight') volume = Math.min(1, volume + 0.05);
						else if (e.key === 'ArrowLeft') volume = Math.max(0, volume - 0.05);
						else return;
						muted = volume === 0;
						e.preventDefault();
					}}
					class="group relative flex h-4 w-20 cursor-pointer touch-none items-center outline-none"
				>
					<div class="h-1.5 w-full overflow-hidden rounded-full bg-edge-strong">
						<div class="h-full rounded-full bg-accent" style="width: {volPct}%"></div>
					</div>
					<div
						class="pointer-events-none absolute top-1/2 size-3 -translate-x-1/2 -translate-y-1/2 rounded-full bg-accent shadow-xs ring-2 ring-surface transition-transform group-hover:scale-110 group-focus-visible:scale-110"
						style="left: {volPct}%"
					></div>
				</div>
			</div>
		</div>
	{/if}
</div>
