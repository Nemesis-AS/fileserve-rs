<script lang="ts">
	import PlayIcon from 'virtual:icons/heroicons/play-solid';
	import PauseIcon from 'virtual:icons/heroicons/pause-solid';
	import SpeakerWave from 'virtual:icons/heroicons/speaker-wave';
	import SpeakerXMark from 'virtual:icons/heroicons/speaker-x-mark';
	import ExpandIcon from 'virtual:icons/heroicons/arrows-pointing-out';
	import CollapseIcon from 'virtual:icons/heroicons/arrows-pointing-in';
	import FilmIcon from 'virtual:icons/heroicons/film';
	import { formatTime as fmt, makeSlider } from '$lib/utils/player';

	let { src, name }: { src: string; name: string } = $props();

	let container = $state<HTMLDivElement>();
	let video = $state<HTMLVideoElement>();

	// Two-way media bindings drive playback; the readonly ones reflect state.
	let paused = $state(true);
	let currentTime = $state(0);
	let duration = $state(0);
	let volume = $state(1);
	let muted = $state(false);
	let rate = $state(1);
	// Svelte's `bind:buffered` yields an array of {start,end}, not a DOM TimeRanges.
	let buffered = $state<{ start: number; end: number }[]>();

	let buffering = $state(false);
	let errored = $state(false);
	let fullscreen = $state(false);
	let pip = $state(false);
	let controlsShown = $state(true);
	let hint = $state('');

	const RATES = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2];

	const progress = $derived(duration ? (currentTime / duration) * 100 : 0);
	const volPct = $derived((muted ? 0 : volume) * 100);
	const bufferedPct = $derived.by(() => {
		if (!buffered || !duration) return 0;
		// End of the range currently playing — what's safe to seek into.
		for (const r of buffered) {
			if (r.start <= currentTime && currentTime <= r.end) return (r.end / duration) * 100;
		}
		const last = buffered.at(-1);
		return last ? (last.end / duration) * 100 : 0;
	});

	// --- controls auto-hide -------------------------------------------------
	let hideTimer: ReturnType<typeof setTimeout> | undefined;
	function poke() {
		controlsShown = true;
		clearTimeout(hideTimer);
		if (!paused) hideTimer = setTimeout(() => (controlsShown = false), 2600);
	}
	// Paused video always shows its controls.
	$effect(() => {
		if (paused) {
			controlsShown = true;
			clearTimeout(hideTimer);
		}
	});

	let hintTimer: ReturnType<typeof setTimeout> | undefined;
	function flash(msg: string) {
		hint = msg;
		clearTimeout(hintTimer);
		hintTimer = setTimeout(() => (hint = ''), 700);
	}

	// --- actions ------------------------------------------------------------
	function togglePlay() {
		if (!video) return;
		if (video.paused) {
			video.play().catch(() => (errored = true));
			flash('Play');
		} else {
			video.pause();
			flash('Pause');
		}
	}

	function skip(delta: number) {
		if (!duration) return;
		currentTime = Math.min(duration, Math.max(0, currentTime + delta));
		flash(`${delta > 0 ? '⏩' : '⏪'} ${Math.abs(delta)}s`);
	}

	function seekToRatio(r: number) {
		if (duration) currentTime = r * duration;
	}

	function changeVolume(delta: number) {
		volume = Math.min(1, Math.max(0, +(volume + delta).toFixed(2)));
		muted = volume === 0;
		flash(`Volume ${Math.round(volume * 100)}%`);
	}

	function stepRate(dir: number) {
		const at = RATES.indexOf(rate);
		const next = Math.min(RATES.length - 1, Math.max(0, (at < 0 ? RATES.indexOf(1) : at) + dir));
		rate = RATES[next];
		flash(`${rate}×`);
	}

	function cycleRate() {
		const at = RATES.indexOf(rate);
		rate = RATES[(at + 1) % RATES.length];
		flash(`${rate}×`);
	}

	async function toggleFullscreen() {
		if (!container) return;
		try {
			if (document.fullscreenElement) await document.exitFullscreen();
			else await container.requestFullscreen();
		} catch {
			/* user gesture / permission — nothing actionable */
		}
	}

	async function togglePip() {
		if (!video || !document.pictureInPictureEnabled) return;
		try {
			if (document.pictureInPictureElement) await document.exitPictureInPicture();
			else await video.requestPictureInPicture();
		} catch {
			/* not allowed right now */
		}
	}

	// The picture-in-picture events aren't in Svelte's typed video props, so bind
	// them imperatively to keep the mute/PiP button state in sync.
	$effect(() => {
		const v = video;
		if (!v) return;
		const enter = () => (pip = true);
		const leave = () => (pip = false);
		v.addEventListener('enterpictureinpicture', enter);
		v.addEventListener('leavepictureinpicture', leave);
		return () => {
			v.removeEventListener('enterpictureinpicture', enter);
			v.removeEventListener('leavepictureinpicture', leave);
		};
	});

	const seek = makeSlider(seekToRatio);
	const vol = makeSlider((r) => {
		volume = r;
		muted = r === 0;
	});

	// --- keyboard (YouTube-style) -------------------------------------------
	function onKey(e: KeyboardEvent) {
		if (e.ctrlKey || e.metaKey || e.altKey) return;
		let handled = true;
		switch (e.key) {
			case ' ':
			case 'k': togglePlay(); break;
			case 'j': skip(-10); break;
			case 'l': skip(10); break;
			case 'ArrowLeft': skip(-5); break;
			case 'ArrowRight': skip(5); break;
			case 'ArrowUp': changeVolume(0.05); break;
			case 'ArrowDown': changeVolume(-0.05); break;
			case 'm': muted = !muted; flash(muted ? 'Muted' : 'Unmuted'); break;
			case 'f': toggleFullscreen(); break;
			case 'i': case 'p': togglePip(); break;
			case '<': stepRate(-1); break;
			case '>': stepRate(1); break;
			case 'Home': currentTime = 0; break;
			case 'End': currentTime = duration; break;
			default:
				if (e.key >= '0' && e.key <= '9') {
					if (duration) currentTime = (Number(e.key) / 10) * duration;
				} else {
					handled = false;
				}
		}
		if (handled) {
			e.preventDefault();
			poke();
		}
	}

	function surfaceClick() {
		togglePlay();
		container?.focus();
	}
</script>

<svelte:document
	onfullscreenchange={() => (fullscreen = document.fullscreenElement === container)}
/>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
	bind:this={container}
	tabindex="0"
	role="application"
	aria-label="Video player: {name}"
	onkeydown={onKey}
	onpointermove={poke}
	ondblclick={toggleFullscreen}
	class="group/player relative flex items-center justify-center overflow-hidden bg-black outline-none
		{fullscreen ? 'h-screen w-screen rounded-none' : 'w-[min(1100px,100%)] rounded-[10px] shadow-xs'}
		{!controlsShown && !paused ? 'cursor-none' : ''}"
>
	<!-- svelte-ignore a11y_media_has_caption -->
	<video
		bind:this={video}
		{src}
		bind:paused
		bind:currentTime
		bind:duration
		bind:volume
		bind:muted
		bind:buffered
		bind:playbackRate={rate}
		onwaiting={() => (buffering = true)}
		onplaying={() => (buffering = false)}
		oncanplay={() => (buffering = false)}
		onerror={() => (errored = true)}
		class="block max-h-full w-full {fullscreen ? 'h-full object-contain' : 'max-h-[78vh]'}"
	></video>

	{#if errored}
		<!-- Unsupported container/codec (e.g. mkv) or decode failure. -->
		<div class="absolute inset-0 grid place-items-center gap-3 bg-black/80 p-6 text-center">
			<FilmIcon class="size-9 text-white/70" />
			<div class="text-[14px] text-white">Can't play this video in the browser</div>
			<div class="max-w-[380px] text-[12.5px] text-white/60">
				The format isn't supported for in-page playback. Download it to watch in another app.
			</div>
		</div>
	{:else}
		<!-- Click-to-toggle surface (sits under the controls & centre affordances). -->
		<button
			type="button"
			aria-label={paused ? 'Play' : 'Pause'}
			onclick={surfaceClick}
			class="absolute inset-0 cursor-pointer"
		></button>

		<!-- Centre affordances -->
		{#if buffering}
			<div
				class="pointer-events-none absolute size-11 animate-spin rounded-full border-[3px] border-white/25 border-t-white"
			></div>
		{:else if paused}
			<div
				class="pointer-events-none absolute grid size-16 place-items-center rounded-full bg-black/55 text-white backdrop-blur-[2px] [&_svg]:size-7"
			>
				<PlayIcon />
			</div>
		{/if}

		{#if hint}
			<div
				class="pointer-events-none absolute rounded-full bg-black/70 px-3.5 py-1.5 text-[13px] font-medium text-white tabular-nums"
			>
				{hint}
			</div>
		{/if}

		<!-- Controls bar -->
		<div
			class="absolute inset-x-0 bottom-0 flex flex-col gap-1.5 bg-gradient-to-t from-black/75 to-transparent px-3 pt-8 pb-2.5 transition-opacity duration-200
				{controlsShown ? 'opacity-100' : 'pointer-events-none opacity-0'}"
		>
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
				class="group/seek relative flex h-4 cursor-pointer touch-none items-center outline-none"
			>
				<div class="relative h-1 w-full overflow-hidden rounded-full bg-white/20">
					<div class="absolute inset-y-0 left-0 bg-white/45" style="width: {bufferedPct}%"></div>
					<div class="absolute inset-y-0 left-0 bg-accent" style="width: {progress}%"></div>
				</div>
				<div
					class="pointer-events-none absolute top-1/2 size-3 -translate-x-1/2 -translate-y-1/2 scale-0 rounded-full bg-accent shadow transition-transform group-hover/seek:scale-100 group-focus-visible/seek:scale-100"
					style="left: {progress}%"
				></div>
			</div>

			<!-- button row -->
			<div class="flex items-center gap-1.5 text-white">
				<button
					type="button"
					onclick={togglePlay}
					aria-label={paused ? 'Play' : 'Pause'}
					class="grid size-8 shrink-0 cursor-pointer place-items-center rounded-md hover:bg-white/15 [&_svg]:size-5"
				>
					{#if paused}<PlayIcon />{:else}<PauseIcon />{/if}
				</button>

				<div class="flex shrink-0 items-center gap-1">
					<button
						type="button"
						onclick={() => (muted = !muted)}
						aria-label={muted ? 'Unmute' : 'Mute'}
						class="grid size-8 shrink-0 cursor-pointer place-items-center rounded-md hover:bg-white/15 [&_svg]:size-[18px]"
					>
						{#if muted || volume === 0}<SpeakerXMark />{:else}<SpeakerWave />{/if}
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
						class="group/vol relative flex h-4 w-16 cursor-pointer touch-none items-center outline-none"
					>
						<div class="h-1 w-full overflow-hidden rounded-full bg-white/25">
							<div class="h-full rounded-full bg-white" style="width: {volPct}%"></div>
						</div>
						<div
							class="pointer-events-none absolute top-1/2 size-3 -translate-x-1/2 -translate-y-1/2 scale-0 rounded-full bg-white shadow transition-transform group-hover/vol:scale-100 group-focus-visible/vol:scale-100"
							style="left: {volPct}%"
						></div>
					</div>
				</div>

				<span class="ml-1 shrink-0 text-[12px] tabular-nums text-white/85">
					{fmt(currentTime)} / {fmt(duration)}
				</span>

				<div class="ml-auto flex shrink-0 items-center gap-1.5">
					<button
						type="button"
						onclick={cycleRate}
						aria-label="Playback speed"
						class="h-7 shrink-0 cursor-pointer rounded-md px-2 text-[12.5px] font-medium tabular-nums hover:bg-white/15"
					>
						{rate}×
					</button>
					<button
						type="button"
						onclick={togglePip}
						aria-label="Picture in picture"
						class="grid size-8 shrink-0 cursor-pointer place-items-center rounded-md hover:bg-white/15 {pip
							? 'text-accent'
							: ''}"
					>
						<!-- No heroicon for PiP; small inline glyph. -->
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
							<rect
								x="3" y="4" width="18" height="15" rx="2"
								stroke="currentColor" stroke-width="1.7"
							/>
							<rect x="12" y="11" width="7" height="5.5" rx="1" fill="currentColor" />
						</svg>
					</button>
					<button
						type="button"
						onclick={toggleFullscreen}
						aria-label={fullscreen ? 'Exit fullscreen' : 'Fullscreen'}
						class="grid size-8 shrink-0 cursor-pointer place-items-center rounded-md hover:bg-white/15 [&_svg]:size-[18px]"
					>
						{#if fullscreen}<CollapseIcon />{:else}<ExpandIcon />{/if}
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>
