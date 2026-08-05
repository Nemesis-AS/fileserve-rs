<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { authStore } from '$lib/stores/auth.svelte';
	import Icon from '$lib/components/Icon.svelte';

	const expiresAt = $derived(authStore.user?.demoExpiresAt ?? null);

	let now = $state(Date.now());

	// The interval must be torn down on unmount, or it survives navigation and
	// keeps ticking against a user that is no longer signed in.
	$effect(() => {
		if (!expiresAt) return;
		const id = setInterval(() => (now = Date.now()), 1000);
		return () => clearInterval(id);
	});

	const msLeft = $derived(expiresAt ? new Date(expiresAt).getTime() - now : 0);

	const urgent = $derived(msLeft > 0 && msLeft < 5 * 60 * 1000);

	const countdown = $derived.by(() => {
		if (msLeft <= 0) return 'expired';
		const total = Math.floor(msLeft / 1000);
		const h = Math.floor(total / 3600);
		const m = Math.floor((total % 3600) / 60);
		const s = total % 60;
		// Switch to a ticking clock in the last few minutes, where seconds
		// actually matter to someone deciding whether to save their work.
		if (h > 0) return `${h}h ${m}m`;
		if (m >= 5) return `${m}m`;
		return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
	});

	// A wall of 401s is a worse ending than a clean bounce to the login page.
	$effect(() => {
		if (expiresAt && msLeft <= 0) {
			void authStore.logout();
			goto(resolve('/login'));
		}
	});

	async function startOver() {
		await authStore.logout();
		goto(resolve('/login'));
	}
</script>

{#if expiresAt}
	<div
		class="flex shrink-0 flex-wrap items-center gap-x-2 gap-y-1 border-b border-edge bg-[color-mix(in_oklab,var(--color-warn)_12%,transparent)] px-4 py-2 text-[12.5px] text-ink"
	>
		<Icon name="Info" size={14} />
		<span class="font-medium">Demo</span>
		<span class="text-ink-muted">
			Files here are deleted when this session ends. Don't upload anything you want to keep.
		</span>
		<span class="ml-auto flex items-center gap-3">
			<span class={urgent ? 'font-semibold text-danger' : 'text-ink-muted'}>
				{countdown} left
			</span>
			<button
				type="button"
				onclick={startOver}
				class="cursor-pointer rounded-md border border-edge px-2 py-0.5 text-[11.5px] font-medium transition-colors hover:bg-row-hover"
			>
				Start over
			</button>
		</span>
	</div>
{/if}
