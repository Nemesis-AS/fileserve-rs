<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { getSettings, updateSettings } from '$lib/services/settings';
	import { Page, PageHead } from '$lib/components/ui/page/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Section } from '$lib/components/ui/section/index.js';
	import { Field } from '$lib/components/ui/field/index.js';
	import { Input } from '$lib/components/ui/input/index.js';

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
</Page>
