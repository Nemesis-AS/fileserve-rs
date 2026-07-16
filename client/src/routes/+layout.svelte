<script lang="ts">
	import './layout.css';
	import { browser } from '$app/environment';
	import { prefs } from '$lib/stores/prefs.svelte';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();

	$effect(() => {
		if (!browser) return;
		document.documentElement.setAttribute('data-theme', prefs.dark ? 'dark' : 'light');
		document.documentElement.setAttribute('data-density', prefs.density);
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
