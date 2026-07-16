<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { authStore } from '$lib/stores/auth.svelte';
	import { login } from '$lib/services/auth';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';

	let username = $state('alex');
	let password = $state('');
	let remember = $state(true);
	let error = $state('');
	let loading = $state(false);

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		loading = true;
		try {
			const { user } = await login(username, password || 'password');
			authStore.login(user);
			goto(resolve('/files/my'));
		} catch (err) {
			error = err instanceof Error ? err.message : 'Sign in failed';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex h-full items-center justify-center">
	<ThemeToggle />

	<form
		onsubmit={handleSubmit}
		class="w-full max-w-90 rounded-xl border border-edge bg-surface p-6 shadow-md"
	>
		<h1 class="text-lg font-semibold">Sign in</h1>
		<p class="mb-6 text-sm text-ink-muted">Welcome back. Use your account on this server.</p>

		{#if error}
			<div class="mb-4 rounded-lg bg-danger-soft px-3 py-2 text-sm text-danger">
				{error}
			</div>
		{/if}

		<div class="mb-3 flex flex-col gap-1.5">
			<label for="username" class="text-xs font-medium text-ink">Username</label>
			<input
				id="username"
				class="w-full rounded-lg border border-edge-strong bg-surface px-2.5 py-2 text-sm text-ink shadow-md outline-0 transition-colors duration-150 focus:border-accent focus:shadow-accent-soft"
				autocomplete="username"
				bind:value={username}
			/>
		</div>

		<div class="mb-3 flex flex-col gap-1.5">
			<label for="password" class="text-xs font-medium text-ink">Password</label>
			<input
				id="password"
				class="w-full rounded-lg border border-edge-strong bg-surface px-2.5 py-2 text-sm text-ink shadow-md outline-0 transition-colors duration-150 focus:border-accent focus:shadow-accent-soft"
				type="password"
				autocomplete="current-password"
				bind:value={password}
			/>
		</div>

		<div class="mb-4 flex items-center justify-between">
			<label class="flex cursor-pointer items-center gap-2 text-xs">
				<input type="checkbox" class="accent-accent" bind:checked={remember} />
				Keep me signed in
			</label>
		</div>

		<button
			type="submit"
			class="inline-flex w-full cursor-pointer items-center justify-center gap-1.5 rounded-lg bg-ink px-3 py-2 text-sm font-medium whitespace-nowrap text-surface transition-opacity duration-100 hover:opacity-90"
			disabled={loading}>{loading ? 'Signing in…' : 'Sign in'}</button
		>

		<p class="mt-4 text-center text-xs text-ink-faint">
			Accounts are managed by the server admin.<br />
			Need access? Ask your admin.
		</p>
	</form>
</div>
