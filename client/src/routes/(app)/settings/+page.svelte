<script lang="ts">
	import { goto } from '$app/navigation';
	import { fly } from 'svelte/transition';
	import TopBar from '$lib/components/TopBar.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { changePassword, updateProfile } from '$lib/services/users';
	import type { Density, ViewMode } from '$lib/types';
	import { Page, PageHead } from '$lib/components/ui/page/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Section } from '$lib/components/ui/section/index.js';
	import { Field } from '$lib/components/ui/field/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { tv } from '$lib/components/ui/tv.js';

	let tab = $state('profile');
	let name = $state(authStore.user?.name ?? '');
	let pwOld = $state('');
	let pwNew = $state('');
	let pwNew2 = $state('');
	let saved = $state('');

	function flashSaved(msg: string) {
		saved = msg;
		setTimeout(() => (saved = ''), 2000);
	}

	async function saveProfile(e: SubmitEvent) {
		e.preventDefault();
		try {
			const updated = await updateProfile({ name });
			authStore.updateUser(updated);
			flashSaved('Profile saved');
		} catch (err) {
			toastStore.show(err instanceof Error ? err.message : 'Could not save profile');
		}
	}

	async function savePassword(e: SubmitEvent) {
		e.preventDefault();
		if (pwNew !== pwNew2) {
			toastStore.show("New passwords don't match");
			return;
		}
		try {
			await changePassword(pwOld, pwNew);
			pwOld = ''; pwNew = ''; pwNew2 = '';
			flashSaved('Password updated');
		} catch (err) {
			toastStore.show(err instanceof Error ? err.message : 'Could not update password');
		}
	}

	const DENSITY_OPTIONS: { value: Density; label: string }[] = [
		{ value: 'compact', label: 'Compact' },
		{ value: 'cozy', label: 'Cozy' },
		{ value: 'comfy', label: 'Comfortable' },
	];

	/**
	 * `.settings__tabs button` — a vertical text list, not the segmented control the file
	 * browser uses, so it stays local rather than being forced into <Segmented>.
	 */
	const tabButton = tv({
		base: 'cursor-pointer rounded-[7px] border-0 bg-transparent px-2.5 py-[7px] text-left font-system text-[13.5px] text-ink-muted transition-[background-color,color] duration-100 hover:bg-row-hover hover:text-ink data-[active=1]:bg-row-active data-[active=1]:font-medium data-[active=1]:text-accent-ink'
	});

	/** The small 28px buttons used throughout preferences/sessions. */
	const SMALL = 'h-7 text-[12px]';

	/** .prefs-row */
	const PREFS_ROW =
		'flex items-center justify-between border-b border-edge py-2 text-[13px] last:border-b-0';

	const TABS: [string, string][] = [
		['profile', 'Profile'],
		['security', 'Security'],
		['prefs', 'Preferences']
	];
</script>

<TopBar
	crumbs={['Settings']}
	dark={prefs.dark}
	onToggleDark={() => (prefs.dark = !prefs.dark)}
	user={authStore.user!}
	onLogout={() => {
		authStore.logout();
		goto('/login');
	}}
/>

<Page class="max-w-[760px]">
	<PageHead title="Settings" sub="Manage your account on this server">
		{#if saved}
			<span
				class="ml-auto inline-flex animate-fade-in items-center gap-[5px] text-[12px] font-medium text-ok [&_svg]:size-[13px]"
				in:fly={{ y: -4, duration: 150 }}
			>
				<Icon name="Check" size={13} />
				{saved}
			</span>
		{/if}
	</PageHead>

	<!-- .settings -->
	<div class="grid grid-cols-[168px_1fr] items-start gap-6">
		<nav class="sticky top-0 flex flex-col gap-0.5">
			{#each TABS as [id, label]}
				<button class={tabButton()} data-active={tab === id ? '1' : '0'} onclick={() => (tab = id)}>
					{label}
				</button>
			{/each}
		</nav>

		<div class="flex flex-col gap-[18px]">
			{#if tab === 'profile'}
				<form onsubmit={saveProfile} class="contents">
					<Section label="Account">
						<div class="flex gap-3">
							<Field label="Full name" class="mb-0 flex-1">
								<Input bind:value={name} />
							</Field>
							<Field label="Username" hint="Contact your admin to change." class="mb-0 flex-1">
								<Input value={authStore.user?.username} disabled />
							</Field>
						</div>
					</Section>
					<div><Button type="submit">Save changes</Button></div>
				</form>
			{:else if tab === 'security'}
				<form onsubmit={savePassword} class="contents">
					<Section label="Change password">
						<Field label="Current password" class="mb-0">
							<Input type="password" bind:value={pwOld} required />
						</Field>
						<Field label="New password" hint="At least 8 characters." class="mb-0">
							<Input type="password" bind:value={pwNew} required minlength={8} />
						</Field>
						<Field label="Confirm new password" class="mb-0">
							<Input type="password" bind:value={pwNew2} required />
						</Field>
					</Section>

					<div><Button type="submit">Update password</Button></div>
				</form>
			{:else if tab === 'prefs'}
				<Section label="Appearance">
					<div class={PREFS_ROW}>
						<span>Theme</span>
						<div class="flex items-center gap-2">
							<span class="text-[13px] text-ink-muted">{prefs.dark ? 'Dark' : 'Light'}</span>
							<Button variant="ghost" class={SMALL} onclick={() => (prefs.dark = !prefs.dark)}>
								<Icon name={prefs.dark ? 'Sun' : 'Moon'} size={13} />
								{prefs.dark ? 'Switch to light' : 'Switch to dark'}
							</Button>
						</div>
					</div>

					<div class={PREFS_ROW}>
						<span>Row density</span>
						<div class="flex gap-1.5">
							{#each DENSITY_OPTIONS as opt}
								<Button
									variant={prefs.density === opt.value ? 'solid' : 'ghost'}
									class={SMALL}
									onclick={() => (prefs.density = opt.value)}
								>
									{opt.label}
								</Button>
							{/each}
						</div>
					</div>

					<div class={PREFS_ROW}>
						<span>Default view</span>
						<div class="flex gap-1.5">
							{#each [['list', 'List'], ['grid', 'Grid']] as [v, l]}
								<Button
									variant={prefs.view === v ? 'solid' : 'ghost'}
									class={SMALL}
									onclick={() => (prefs.view = v as ViewMode)}
								>
									{l}
								</Button>
							{/each}
						</div>
					</div>

					<div class={PREFS_ROW}>
						<span>Sidebar</span>
						<Button
							variant="ghost"
							class={SMALL}
							onclick={() => (prefs.showSidebar = !prefs.showSidebar)}
						>
							{prefs.showSidebar ? 'Hide sidebar' : 'Show sidebar'}
						</Button>
					</div>
				</Section>
			{/if}
		</div>
	</div>
</Page>
