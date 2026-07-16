<script>
	import SunIcon from 'virtual:icons/heroicons/sun';
	import MoonIcon from 'virtual:icons/heroicons/moon';

	import { prefs } from '$lib/stores/prefs.svelte';
</script>

<button
	class="absolute top-4 right-4 cursor-pointer rounded-md p-2 transition-colors hover:bg-white/10"
	onclick={() => (prefs.dark = !prefs.dark)}
	aria-label="Toggle Theme"
>
	{#if prefs.dark}
		<SunIcon />
	{:else}
		<MoonIcon />
	{/if}
</button>
