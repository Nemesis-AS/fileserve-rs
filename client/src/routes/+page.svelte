<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { browser } from '$app/environment';
	import { authStore } from '$lib/stores/auth.svelte';

	if (browser) {
		if (authStore.isLoggedIn) goto(resolve('/files/my'), { replaceState: true });
		else goto(resolve('/login'), { replaceState: true });
	}
</script>
