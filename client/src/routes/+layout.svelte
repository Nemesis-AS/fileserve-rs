<script lang="ts">
	import './layout.css';
	import { browser } from '$app/environment';
	import { prefs } from '$lib/stores/prefs.svelte';

	let { children } = $props();

	const DESCRIPTION =
		'Your files, served. A self-hosted file server with folders, uploads, previews, and trash.';

	$effect(() => {
		if (!browser) return;
		document.documentElement.setAttribute('data-theme', prefs.dark ? 'dark' : 'light');
		document.documentElement.setAttribute('data-density', prefs.density);
	});
</script>

<svelte:head>
	<title>fileserve.rs</title>
	<meta name="description" content={DESCRIPTION} />
	<meta property="og:type" content="website" />
	<meta property="og:site_name" content="fileserve.rs" />
	<meta property="og:title" content="fileserve.rs" />
	<meta property="og:description" content={DESCRIPTION} />
	<meta property="og:image" content="/icon-512.png" />
	<meta name="twitter:card" content="summary" />
	<meta name="twitter:title" content="fileserve.rs" />
	<meta name="twitter:description" content={DESCRIPTION} />
	<meta name="twitter:image" content="/icon-512.png" />
</svelte:head>

{@render children()}
