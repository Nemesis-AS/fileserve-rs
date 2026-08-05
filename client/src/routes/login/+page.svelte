<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { authStore } from '$lib/stores/auth.svelte';
	import { serverConfig } from '$lib/stores/serverConfig.svelte';
	import { login, startDemo } from '$lib/services/auth';
	import { fmtSize } from '$lib/utils/file';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import Logo from '$lib/components/Logo.svelte';

	let username = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);
	let demoLoading = $state(false);

	const cfg = $derived(serverConfig.config);

	const demoBlurb = $derived.by(() => {
		const quota = cfg.demoQuotaBytes ? fmtSize(cfg.demoQuotaBytes) : '';
		const hours = cfg.demoTtlMinutes ? Math.round(cfg.demoTtlMinutes / 60) : 0;
		const window = hours >= 1 ? `${hours} hour${hours === 1 ? '' : 's'}` : `${cfg.demoTtlMinutes} minutes`;
		return `Creates a temporary account. ${quota} of storage, deleted after ${window}.`;
	});

	async function handleDemo() {
		error = '';
		demoLoading = true;
		try {
			const { user } = await startDemo();
			authStore.login(user);
			goto(resolve('/files/my'));
		} catch (err) {
			error = err instanceof Error ? err.message : 'Could not start the demo';
		} finally {
			demoLoading = false;
		}
	}

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		loading = true;
		try {
			const { user } = await login(username, password);
			authStore.login(user);
			goto(resolve('/files/my'));
		} catch (err) {
			error = err instanceof Error ? err.message : 'Sign in failed';
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Sign in · fileserve.rs</title>
</svelte:head>

<div class="flex h-full items-center justify-center">
	<ThemeToggle />

	<form
		onsubmit={handleSubmit}
		class="w-full max-w-90 rounded-xl border border-edge bg-surface p-6 shadow-md"
	>
		<div class="mb-6 flex flex-col items-center gap-3 text-center">
			<Logo size={40} />
			<span class="text-[19px] font-bold tracking-[-0.035em] text-ink">
				fileserve<span
					class="mx-0.5 inline-block size-1.25 rounded-full bg-accent align-baseline"
				></span><span class="font-mono text-[16px] font-semibold text-accent-ink">rs</span>
			</span>
		</div>

		<h1 class="text-lg font-semibold">Sign in</h1>
		<p class="mb-6 text-sm text-ink-muted">Welcome back. Use your account on this server.</p>

		{#if error}
			<div class="mb-4 rounded-lg bg-danger-soft px-3 py-2 text-sm text-danger">
				{error}
			</div>
		{/if}

		{#if cfg.demo}
			<div class="mb-5 rounded-lg border border-edge bg-sunken p-3.5">
				<button
					type="button"
					onclick={handleDemo}
					disabled={demoLoading}
					class="inline-flex w-full cursor-pointer items-center justify-center gap-1.5 rounded-lg bg-accent px-3 py-2 text-sm font-medium whitespace-nowrap text-accent-ink transition-opacity duration-100 hover:opacity-90 disabled:opacity-60"
				>
					{demoLoading ? 'Starting…' : 'Try the demo'}
				</button>
				<p class="mt-2 text-center text-[11.5px] leading-relaxed text-ink-faint">
					{demoBlurb}<br />Don't upload anything you want to keep.
				</p>
			</div>

			<div class="mb-5 flex items-center gap-3 text-[11px] text-ink-faint">
				<span class="h-px flex-1 bg-edge"></span>or sign in<span
					class="h-px flex-1 bg-edge"
				></span>
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

		<div class="mb-4"></div>

		<button
			type="submit"
			class="inline-flex w-full cursor-pointer items-center justify-center gap-1.5 rounded-lg bg-ink px-3 py-2 text-sm font-medium whitespace-nowrap text-surface transition-opacity duration-100 hover:opacity-90"
			disabled={loading}>{loading ? 'Signing in…' : 'Sign in'}</button
		>

		{#if !cfg.demo}
			<p class="mt-4 text-center text-xs text-ink-faint">
				Accounts are managed by the server admin.<br />
				Need access? Ask your admin.
			</p>
		{/if}
	</form>
</div>
