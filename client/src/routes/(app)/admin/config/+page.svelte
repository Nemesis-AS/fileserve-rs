<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { getSettings, updateSettings } from '$lib/services/settings';
	import {
		checkForUpdate,
		getInstallProgress,
		getUpdateStatus,
		pingVersion,
		restartServer,
		startInstall,
		type RestartMode,
		type UpdateStatus
	} from '$lib/services/updates';
	import { fmtDate, fmtSize } from '$lib/utils/file';
	import { Page, PageHead } from '$lib/components/ui/page/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Section } from '$lib/components/ui/section/index.js';
	import { Field } from '$lib/components/ui/field/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Meter } from '$lib/components/ui/meter/index.js';
	import { Modal } from '$lib/components/ui/modal/index.js';

	let loaded = $state(false);
	let saving = $state(false);
	let storagePath = $state('');
	let maxUploadGB = $state(5);
	let defaultQuotaGB = $state(20);

	onMount(async () => {
		try {
			const s = await getSettings();
			storagePath = s.storagePath;
			maxUploadGB = s.maxUploadGB;
			defaultQuotaGB = s.defaultQuotaGB;
			loaded = true;
		} catch (e) {
			toastStore.show(e instanceof Error ? e.message : 'Failed to load settings');
		}
	});

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		saving = true;
		try {
			await updateSettings({ storagePath: storagePath.trim(), maxUploadGB, defaultQuotaGB });
			toastStore.show('Settings saved');
		} catch (err) {
			toastStore.show(err instanceof Error ? err.message : 'Failed to save settings');
		} finally {
			saving = false;
		}
	}

	// Loaded independently of the settings above: a settings failure leaves this
	// page on "Loading…" forever, and the update banner shouldn't go with it.
	const RESTART_GRACE_MS = 2000;
	const RESTART_POLL_MS = 1200;
	const RESTART_TIMEOUT_MS = 90_000;
	const PROGRESS_POLL_MS = 700;

	let status = $state<UpdateStatus | null>(null);
	let updatesError = $state<string | null>(null);
	let checking = $state(false);
	let installing = $state(false);
	let confirmingRestart = $state(false);
	let restarting = $state(false);
	let restartMode = $state<RestartMode>('spawn');
	let restartElapsed = $state(0);
	/** The version we were on when "Restart now" was clicked. */
	let versionBeforeRestart = $state<string | null>(null);
	let rolledBackFrom = $state<string | null>(null);
	let restartTimedOut = $state(false);

	const phase = $derived(status?.install.phase ?? 'idle');
	const busy = $derived(phase === 'downloading' || phase === 'verifying' || phase === 'applying');
	const percent = $derived(
		status && status.install.totalBytes > 0
			? Math.round((status.install.downloadedBytes / status.install.totalBytes) * 100)
			: 0
	);

	onMount(loadUpdateStatus);

	async function loadUpdateStatus() {
		try {
			status = await getUpdateStatus();
			updatesError = null;
		} catch (e) {
			updatesError = e instanceof Error ? e.message : 'Could not load update status';
		}
	}

	async function handleCheck() {
		checking = true;
		rolledBackFrom = null;
		restartTimedOut = false;
		try {
			status = await checkForUpdate();
			updatesError = null;
		} catch (e) {
			updatesError = e instanceof Error ? e.message : 'Update check failed';
		} finally {
			checking = false;
		}
	}

	async function handleInstall() {
		if (!status?.latest) return;
		installing = true;
		try {
			const install = await startInstall(status.latest.version);
			status = { ...status, install };
		} catch (e) {
			toastStore.show(e instanceof Error ? e.message : 'Could not start the install');
			// The server may have refused because our view is stale.
			await loadUpdateStatus();
		} finally {
			installing = false;
		}
	}

	async function handleRestart() {
		confirmingRestart = false;
		versionBeforeRestart = status?.currentVersion ?? null;
		restartElapsed = 0;
		restartTimedOut = false;
		rolledBackFrom = null;
		try {
			restartMode = await restartServer();
			restarting = true;
		} catch (e) {
			toastStore.show(e instanceof Error ? e.message : 'Could not restart the server');
		}
	}

	// Self-rescheduling setTimeout, not setInterval, so slow responses can't
	// stack polls.
	$effect(() => {
		if (!busy) return;

		let cancelled = false;
		let timer: ReturnType<typeof setTimeout>;

		const tick = async () => {
			try {
				const install = await getInstallProgress();
				if (!cancelled && status) status = { ...status, install };
			} catch {
				/* keep the last known progress; the next tick retries */
			}
			if (!cancelled) timer = setTimeout(tick, PROGRESS_POLL_MS);
		};

		timer = setTimeout(tick, PROGRESS_POLL_MS);
		return () => {
			cancelled = true;
			clearTimeout(timer);
		};
	});

	$effect(() => {
		if (!restarting) return;

		let cancelled = false;
		let timer: ReturnType<typeof setTimeout>;
		const started = Date.now();

		const tick = async () => {
			const version = await pingVersion();
			if (cancelled) return;

			// Only a *different* build means success: during the grace period
			// the outgoing process may still be answering.
			if (version && version !== versionBeforeRestart) {
				restarting = false;
				toastStore.show(`Server restarted on v${version}`);
				await loadUpdateStatus();
				return;
			}

			// Back on the version we started from means it rolled back.
			if (
				version &&
				version === versionBeforeRestart &&
				Date.now() - started > RESTART_GRACE_MS * 4
			) {
				restarting = false;
				rolledBackFrom = status?.install.version ?? null;
				await loadUpdateStatus();
				return;
			}

			restartElapsed = Math.floor((Date.now() - started) / 1000);

			if (Date.now() - started > RESTART_TIMEOUT_MS) {
				restarting = false;
				restartTimedOut = true;
				return;
			}

			timer = setTimeout(tick, RESTART_POLL_MS);
		};

		timer = setTimeout(tick, RESTART_GRACE_MS);
		return () => {
			cancelled = true;
			clearTimeout(timer);
		};
	});
</script>

<TopBar
	crumbs={['Admin', 'Configuration']}
	dark={prefs.dark}
	onToggleDark={() => (prefs.dark = !prefs.dark)}
	user={authStore.user!}
	onLogout={() => {
		authStore.logout();
		goto('/login');
	}}
	onSettings={() => goto('/settings')}
/>

<Page class="max-w-[680px]">
	<PageHead title="Configuration" sub="Server-wide settings · admin-managed" />

	<div class="grid gap-[18px]">
		{#if !loaded}
			<p class="text-ink-muted">Loading…</p>
		{:else}
			<form onsubmit={handleSubmit} class="grid gap-[18px]">
				<Section label="Storage">
					<Field
						label="Storage path"
						hint="Where uploads are written. Applies to new uploads only — existing files are not moved."
						class="mb-0"
					>
						<Input bind:value={storagePath} required />
					</Field>
				</Section>

				<Section label="Limits">
					<div class="flex gap-3">
						<Field label="Max upload size" class="mb-0 flex-1">
							<div class="flex items-center gap-3">
								<Input
									type="number"
									min={1}
									max={2000}
									step={1}
									bind:value={maxUploadGB}
									class="w-[110px]"
								/>
								<span class="text-ink-muted">GB</span>
							</div>
						</Field>
						<Field
							label="Default quota"
							hint="Applied to new users when none is specified."
							class="mb-0 flex-1"
						>
							<div class="flex items-center gap-3">
								<Input
									type="number"
									min={0}
									max={2000}
									step={1}
									bind:value={defaultQuotaGB}
									class="w-[110px]"
								/>
								<span class="text-ink-muted">GB</span>
							</div>
						</Field>
					</div>
				</Section>

				<div class="flex gap-2 pt-1">
					<Button type="submit" disabled={saving}>{saving ? 'Saving…' : 'Save changes'}</Button>
				</div>
			</form>
		{/if}

		<!-- Outside the form above: these buttons must not submit the settings. -->
		<Section label="Updates">
			{#if updatesError}
				<p class="flex items-start gap-2 text-[13px] text-ink-muted">
					<Icon name="Info" class="mt-px shrink-0" />
					<span>{updatesError}</span>
				</p>
				<div><Button variant="ghost" onclick={loadUpdateStatus}>Retry</Button></div>
			{:else if !status}
				<p class="text-ink-muted">Loading…</p>
			{:else if restarting}
				<p class="text-[13px] text-ink-muted">
					Restarting… waiting for the server ({restartElapsed}s)
				</p>
				<Meter value={100} size="sm" speed="fast" />
				{#if restartMode === 'exit'}
					<p class="text-[12px] text-ink-faint">
						This server is supervised — it has exited and its service manager should start it again.
						If no restart policy is configured, it will stay down.
					</p>
				{/if}
			{:else}
				<div class="flex flex-wrap items-center gap-3">
					<span class="text-ink">Version <strong>{status.currentVersion}</strong></span>

					{#if !status.enabled}
						<Badge tone="neutral">Self-update disabled</Badge>
					{:else if phase === 'ready'}
						<Badge tone="active" dot>v{status.install.version} installed</Badge>
					{:else if phase === 'failed'}
						<Badge tone="suspended">Update failed</Badge>
					{:else if rolledBackFrom}
						<Badge tone="suspended">Rolled back</Badge>
					{:else if status.latest}
						<Badge tone="admin">v{status.latest.version} available</Badge>
					{:else if !status.checkError}
						<Badge tone="active" dot>Up to date</Badge>
					{/if}

					{#if status.lastChecked && status.enabled}
						<span class="text-[12px] text-ink-faint">
							Checked {fmtDate(status.lastChecked.toISOString())}
						</span>
					{/if}
				</div>

				{#if !status.enabled}
					<p class="text-[12px] text-ink-faint">
						Started with SELF_UPDATE_ENABLED=false. Updates must be installed by hand.
					</p>
				{:else}
					{#if status.checkError}
						<p class="flex items-start gap-2 text-[13px] text-ink-muted">
							<Icon name="Info" class="mt-px shrink-0" />
							<span>{status.checkError}</span>
						</p>
					{/if}

					{#if restartTimedOut}
						<p class="text-[13px] text-warn">
							The server didn't come back within 90 seconds. Check it directly — it may need to be
							started by hand.
						</p>
					{/if}

					{#if rolledBackFrom}
						<p class="text-[13px] text-warn">
							v{rolledBackFrom} failed to start, so the server put v{status.currentVersion}
							back. Check the server's console output for why.
						</p>
					{/if}

					{#if busy}
						<Meter value={percent} size="sm" />
						<p class="text-[13px] text-ink-muted">
							{#if phase === 'downloading'}
								Downloading v{status.install.version} — {fmtSize(status.install.downloadedBytes)} of {fmtSize(
									status.install.totalBytes
								)} ({percent}%)
							{:else if phase === 'verifying'}
								Verifying the download…
							{:else}
								Installing…
							{/if}
						</p>
					{:else if phase === 'ready'}
						<p class="text-[13px] text-ink-muted">
							Restart the server to apply it. The version it replaces is kept until the new one has
							proved it starts.
						</p>
						<div class="flex gap-2">
							<Button onclick={() => (confirmingRestart = true)}>
								<Icon name="Power" /> Restart now
							</Button>
						</div>
					{:else if phase === 'failed'}
						<p class="text-[13px] text-warn">{status.install.error}</p>
						<div class="flex gap-2">
							<Button variant="ghost" onclick={handleInstall} disabled={installing}>
								Try again
							</Button>
						</div>
					{:else if status.latest}
						{#if status.latest.name}
							<p class="text-[13px] text-ink">{status.latest.name}</p>
						{/if}
						{#if status.latest.body}
							<details class="text-[13px]">
								<summary class="cursor-pointer text-ink-muted select-none hover:text-ink">
									Release notes
								</summary>
								<pre
									class="mt-2 max-h-64 overflow-auto rounded-md border border-edge p-3 text-[12px] whitespace-pre-wrap text-ink-muted">{status
										.latest.body}</pre>
							</details>
						{/if}
						<div class="flex flex-wrap items-center gap-2">
							<Button onclick={handleInstall} disabled={installing}>
								<Icon name="Download" />
								{installing ? 'Starting…' : 'Download & install'}
							</Button>
							<Button variant="ghost" onclick={handleCheck} disabled={checking}>
								{checking ? 'Checking…' : 'Check again'}
							</Button>
							<span class="text-[12px] text-ink-faint">
								{fmtSize(status.latest.assetSize)} · checksum verified on download
							</span>
						</div>
						<!-- An absolute URL to GitHub, not an app route, so there is
							 nothing for SvelteKit's resolve() to do here. -->
						<!-- eslint-disable svelte/no-navigation-without-resolve -->
						<a
							class="text-[12px] text-ink-muted underline underline-offset-2 hover:text-ink"
							href={status.latest.htmlUrl}
							target="_blank"
							rel="noreferrer noopener"
						>
							View release on GitHub
						</a>
						<!-- eslint-enable svelte/no-navigation-without-resolve -->
					{:else}
						<div class="flex gap-2">
							<Button variant="ghost" onclick={handleCheck} disabled={checking}>
								<Icon name="Refresh" />
								{checking ? 'Checking…' : 'Check now'}
							</Button>
						</div>
					{/if}
				{/if}
			{/if}
		</Section>
	</div>
</Page>

{#if confirmingRestart}
	<Modal title="Restart the server?" onClose={() => (confirmingRestart = false)}>
		<p class="text-[13px] text-ink-muted">
			The server will be briefly unavailable. Requests already in flight, including uploads, are
			allowed to finish first — so this can take a moment if a large upload is running.
		</p>
		<p class="mt-2 text-[13px] text-ink-muted">
			If the new version fails to start, the server puts the previous one back on its own.
		</p>
		{#snippet footer()}
			<Button variant="ghost" onclick={() => (confirmingRestart = false)}>Cancel</Button>
			<Button onclick={handleRestart}>Restart</Button>
		{/snippet}
	</Modal>
{/if}
