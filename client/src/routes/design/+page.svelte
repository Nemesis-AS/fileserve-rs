<script lang="ts">
	import Icon from '$lib/components/Icon.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import {
		Button,
		IconButton,
		Input,
		Select,
		Field,
		Checkbox,
		Badge,
		Avatar,
		Menu,
		MenuItem,
		MenuSeparator,
		MenuHeader,
		Section,
		Meter,
		Card,
		SearchInput,
		Segmented,
		NavItem,
		Dropzone,
		EmptyState,
		table,
		cell
	} from '$lib/components/ui';
	import { BackButton } from '$lib/components/ui/back-button/index.js';
	import { tv } from '$lib/components/ui/tv.js';

	/**
	 * Living design showcase.
	 *
	 * This page renders the SAME components the app ships, from $lib/components/ui — so it
	 * cannot drift from production the way the old design/ and design-tw/ pages did. Those
	 * each re-implemented every control by hand, which meant three copies of every button
	 * to keep in sync.
	 */

	const SECTIONS = [
		['colors', 'Colors'],
		['foundations', 'Foundations'],
		['components', 'Components']
	] as const;

	const SURFACES = [
		['--bg / surface', 'Page background', 'bg-surface'],
		['--bg-elev / elevated', 'Sidebar, modal footer', 'bg-elevated'],
		['--bg-sunken / sunken', 'Search field, viewer stage', 'bg-sunken'],
		['--border / edge', 'Hairlines, dividers', 'bg-edge'],
		['--border-strong / edge-strong', 'Input border, ghost button', 'bg-edge-strong']
	];

	const INK = [
		['--text / ink', 'Primary text', 'bg-ink'],
		['--text-muted / ink-muted', 'Secondary text, metadata', 'bg-ink-muted'],
		['--text-faint / ink-faint', 'Eyebrow labels, counts', 'bg-ink-faint']
	];

	const SEMANTIC = [
		['--accent / accent', 'Focus ring, quota bar', 'bg-accent'],
		['--accent-soft / accent-soft', 'Active nav, admin badge', 'bg-accent-soft'],
		['--accent-text / accent-ink', 'Active nav text', 'bg-accent-ink'],
		['--danger / danger', 'Destructive actions', 'bg-danger'],
		['--success / ok', 'Upload done, active dot', 'bg-ok'],
		['--warn / warn', 'Suspended, delete audit', 'bg-warn']
	];

	const RADII = [
		['3px', 'Shared pill on file rows'],
		['5px', 'File icon, small controls'],
		['6px', 'Menu item, row action'],
		['7px', 'Button, input, nav item'],
		['8px', 'Toast, viewer image'],
		['10px', 'Card, grid tile, panel'],
		['12px', 'Modal'],
		['999px', 'Badge, quota bar']
	];

	const SHADOWS = [
		['shadow-xs', 'Active segmented button'],
		['shadow-pop', 'Menu, toast, viewer image'],
		['shadow-lift', 'Modal']
	];

	const DENSITY: { value: 'compact' | 'cozy' | 'comfy'; label: string; h: string }[] = [
		{ value: 'compact', label: 'Compact', h: '30px' },
		{ value: 'cozy', label: 'Cozy', h: '40px' },
		{ value: 'comfy', label: 'Comfortable', h: '52px' }
	];

	const sectionHd = tv({
		base: 'mt-12 mb-4 border-b border-edge pb-2.5 text-[10px] font-semibold tracking-[0.08em] text-ink-faint uppercase'
	});
	const eyebrow = tv({
		base: 'mb-3 text-[10px] font-semibold tracking-[0.08em] text-ink-faint uppercase'
	});

	let menuOpen = $state(true);
	let view = $state<'list' | 'grid'>('list');
	let checked = $state(true);
	let dragging = $state(false);
	let text = $state('mountain-lake.jpg');

	const t = table({ variant: 'files' });

	const ROWS = [
		{ name: 'mountain-lake.jpg', ext: 'jpg', size: '3.3 MB', when: '2h ago', color: '#0ea5e9' },
		{ name: 'lease-2026.pdf', ext: 'pdf', size: '1.2 MB', when: '1d ago', color: '#dc2626' },
		{ name: 'backup.py', ext: 'py', size: '12 KB', when: '3w ago', color: '#475569' }
	];

	// Hoisted: `{#each [...] as const as [a, b]}` confuses Svelte's `as` parsing.
	const SWATCH_GROUPS: [string, string[][]][] = [
		['Surfaces', SURFACES],
		['Text', INK],
		['Semantic', SEMANTIC]
	];
</script>

<svelte:head>
	<title>Design System · filer</title>
</svelte:head>

<div class="min-h-full bg-surface font-system text-[13.5px] leading-[1.45] text-ink antialiased">
	<!-- Top nav -->
	<div class="sticky top-0 z-10 flex h-12 items-center gap-4 border-b border-edge bg-surface px-8">
		<div class="flex items-center gap-[9px] text-[14px] font-semibold tracking-tight">
			<div class="grid size-[22px] shrink-0 place-items-center rounded-md bg-ink text-surface">
				<Icon name="Files" size={13} />
			</div>
			filer
		</div>
		<span class="text-edge-strong">·</span>
		<span class="text-[13.5px] text-ink-muted">Design System</span>
		<span
			class="ml-2 rounded-full bg-accent-soft px-[7px] py-px text-[10px] font-semibold text-accent-ink"
		>
			Live components
		</span>

		<div class="ml-auto flex items-center gap-4">
			{#each SECTIONS as [id, label]}
				<a href="#{id}" class="text-[13px] text-ink-muted hover:text-ink">{label}</a>
			{/each}
			<IconButton title="Toggle theme" onclick={() => (prefs.dark = !prefs.dark)}>
				<Icon name={prefs.dark ? 'Sun' : 'Moon'} size={16} />
			</IconButton>
		</div>
	</div>

	<div class="mx-auto max-w-[1100px] px-8 pb-24">
		<!-- ── Colors ─────────────────────────────────────────── -->
		<h2 id="colors" class={sectionHd()}>Colors</h2>
		<p class="mb-6 max-w-[640px] text-[13px] text-ink-muted">
			Every colour is a CSS custom property in
			<code class="font-code text-[12px]">routes/layout.css</code>, re-exported to Tailwind through
			<code class="font-code text-[12px]">@theme inline</code>. Dark mode swaps those variables under
			<code class="font-code text-[12px]">[data-theme='dark']</code>, so the utilities below need no
			<code class="font-code text-[12px]">dark:</code> variant to change.
		</p>

		<div class="grid grid-cols-1 gap-4 md:grid-cols-3">
			{#each SWATCH_GROUPS as [label, rows]}
				<Card>
					<div class={eyebrow()}>{label}</div>
					<div class="flex flex-col gap-2.5">
						{#each rows as [token, desc, swatch]}
							<div class="flex items-center gap-2.5">
								<div class="size-7 shrink-0 rounded-md border border-edge {swatch}"></div>
								<div class="min-w-0">
									<div class="truncate font-code text-[11.5px]">{token}</div>
									<div class="truncate text-[11.5px] text-ink-muted">{desc}</div>
								</div>
							</div>
						{/each}
					</div>
				</Card>
			{/each}
		</div>

		<!-- ── Foundations ────────────────────────────────────── -->
		<h2 id="foundations" class={sectionHd()}>Foundations</h2>

		<div class="grid grid-cols-1 gap-4 md:grid-cols-3">
			<Card>
				<div class={eyebrow()}>Radius</div>
				<div class="flex flex-col gap-2">
					{#each RADII as [px, desc]}
						<div class="flex items-center gap-2.5">
							<div
								class="size-7 shrink-0 border border-edge-strong bg-sunken"
								style="border-radius: {px};"
							></div>
							<div class="min-w-0">
								<div class="font-code text-[11.5px]">{px}</div>
								<div class="truncate text-[11.5px] text-ink-muted">{desc}</div>
							</div>
						</div>
					{/each}
				</div>
			</Card>

			<Card>
				<div class={eyebrow()}>Elevation</div>
				<div class="flex flex-col gap-4 pt-1">
					{#each SHADOWS as [cls, desc]}
						<div class="flex items-center gap-3">
							<div class="size-10 shrink-0 rounded-lg bg-surface {cls}"></div>
							<div>
								<div class="font-code text-[11.5px]">{cls}</div>
								<div class="text-[11.5px] text-ink-muted">{desc}</div>
							</div>
						</div>
					{/each}
				</div>
			</Card>

			<Card>
				<div class={eyebrow()}>Density</div>
				<p class="mb-3 text-[11.5px] text-ink-muted">
					Sets <code class="font-code">--row-h</code> on
					<code class="font-code">&lt;html&gt;</code>. The
					<code class="font-code">h-row</code> utility tracks it with no JS — this control is live.
				</p>
				<div class="flex flex-col gap-1.5">
					{#each DENSITY as d}
						<Button
							variant={prefs.density === d.value ? 'solid' : 'ghost'}
							class="h-7 justify-between text-[12px]"
							onclick={() => (prefs.density = d.value)}
						>
							{d.label}<span class="font-code opacity-70">{d.h}</span>
						</Button>
					{/each}
				</div>
			</Card>
		</div>

		<!-- ── Components ─────────────────────────────────────── -->
		<h2 id="components" class={sectionHd()}>Components</h2>

		<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
			<Card>
				<div class={eyebrow()}>Button</div>
				<div class="flex flex-wrap items-center gap-2">
					<Button>Upload</Button>
					<Button variant="ghost">Cancel</Button>
					<Button variant="danger">Delete</Button>
					<Button size="lg">Sign in</Button>
					<Button><Icon name="Upload" size={14} />With icon</Button>
				</div>
			</Card>

			<Card>
				<div class={eyebrow()}>Icon button</div>
				<div class="flex flex-wrap items-center gap-2">
					<IconButton title="Default"><Icon name="Moon" size={16} /></IconButton>
					<IconButton variant="row" size="sm" title="Row action">
						<Icon name="Eye" size={14} />
					</IconButton>
					<IconButton size="xs" title="Remove"><Icon name="Close" size={12} /></IconButton>
					<div class="grid size-[46px] place-items-center rounded-md bg-ink-muted">
						<IconButton variant="overlay" size="sm" title="Overlay">
							<Icon name="Download" size={14} />
						</IconButton>
					</div>
				</div>
			</Card>

			<Card>
				<div class={eyebrow()}>Badge</div>
				<div class="flex flex-wrap items-center gap-2">
					<Badge>Neutral</Badge>
					<Badge tone="admin">Admin</Badge>
					<Badge tone="active" dot>Active</Badge>
					<Badge tone="suspended" dot>Suspended</Badge>
				</div>
			</Card>

			<Card>
				<div class={eyebrow()}>Avatar</div>
				<div class="flex flex-wrap items-center gap-3">
					<Avatar name="Alex Chen" />
					<Avatar name="Sam Reyes" size="md" />
					<Avatar name="Leo Park" size="lg" />
				</div>
			</Card>

			<Card>
				<div class={eyebrow()}>Form</div>
				<Field label="File name" hint="Extensions are preserved.">
					<Input bind:value={text} />
				</Field>
				<Field label="Large" class="mb-3">
					<Input size="lg" value="Rename input" />
				</Field>
				<Field label="Role" class="mb-3">
					<Select value="admin">
						<option value="user">User</option>
						<option value="admin">Admin</option>
					</Select>
				</Field>
				<Field label="Disabled" class="mb-3">
					<Input value="alex" disabled />
				</Field>
				<Checkbox bind:checked>Make these files public</Checkbox>
			</Card>

			<Card>
				<div class={eyebrow()}>Search · Segmented · Filter select</div>
				<div class="flex flex-col gap-3">
					<SearchInput placeholder="Search files…" kbd="/" class="max-w-[320px]" />
					<Segmented
						value={view}
						options={[
							{ value: 'list', label: 'List view' },
							{ value: 'grid', label: 'Grid view' }
						]}
						onchange={(v) => (view = v)}
						class="self-start"
					>
						{#snippet item(v)}
							{#if v === 'list'}
								<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
									<path d="M3 4h10M3 8h10M3 12h10" />
								</svg>
							{:else}
								<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6">
									<rect x="2.5" y="2.5" width="4.5" height="4.5" rx="1" />
									<rect x="9" y="2.5" width="4.5" height="4.5" rx="1" />
									<rect x="2.5" y="9" width="4.5" height="4.5" rx="1" />
									<rect x="9" y="9" width="4.5" height="4.5" rx="1" />
								</svg>
							{/if}
						{/snippet}
					</Segmented>
					<Select variant="filter" value="all" class="self-start">
						<option value="all">All events</option>
						<option value="upload">Upload</option>
					</Select>
					<BackButton class="self-start">Back</BackButton>
				</div>
			</Card>

			<Card>
				<div class={eyebrow()}>Meter</div>
				<div class="flex flex-col gap-4">
					<div>
						<div class="mb-1.5 flex justify-between text-[12px] text-ink-muted">
							<span>Storage</span><span><b class="font-medium text-ink">84.2</b> / 200 GB</span>
						</div>
						<Meter value={42} />
					</div>
					<div>
						<div class="mb-1 text-[11.5px] text-ink-muted">Upload progress (xs · sharp)</div>
						<Meter value={68} size="xs" radius="sharp" speed="fast" />
					</div>
					<div>
						<div class="mb-1 text-[11.5px] text-ink-muted">Failed upload</div>
						<Meter value={38} size="xs" radius="sharp" speed="fast" color="var(--danger)" />
					</div>
				</div>
			</Card>

			<Card>
				<div class={eyebrow()}>Nav item</div>
				<div class="flex flex-col gap-px">
					<NavItem active count={30}>
						{#snippet icon()}<Icon name="Folder" size={16} />{/snippet}
						My Files
					</NavItem>
					<NavItem count={7}>
						{#snippet icon()}<Icon name="Public" size={16} />{/snippet}
						Public
					</NavItem>
					<NavItem>
						{#snippet icon()}<Icon name="TrashBin" size={16} />{/snippet}
						Trash
					</NavItem>
				</div>
			</Card>

			<Card class="relative min-h-[280px]">
				<div class={eyebrow()}>Menu</div>
				<Button variant="ghost" class="mb-2" onclick={() => (menuOpen = !menuOpen)}>
					Toggle menu
				</Button>
				<div class="relative">
					{#if menuOpen}
						<Menu class="top-0 left-0">
							<MenuHeader title="Alex Chen">alex@home.lan</MenuHeader>
							<MenuSeparator />
							<MenuItem><Icon name="Settings" size={14} />Settings</MenuItem>
							<MenuItem><Icon name="Lock" size={14} />Change password</MenuItem>
							<MenuSeparator />
							<MenuItem danger><Icon name="LogOut" size={14} />Sign out</MenuItem>
						</Menu>
					{/if}
				</div>
			</Card>

			<Card>
				<div class={eyebrow()}>Dropzone</div>
				<Dropzone
					active={dragging}
					hint="Single files up to 5 GB · multiple selections OK"
					onclick={() => (dragging = !dragging)}
				>
					<Icon name="Upload" size={28} class="mx-auto" />
					<div class="text-[13px] font-medium text-ink">
						Drop files here or <span class="text-accent-ink">browse</span>
					</div>
				</Dropzone>
				<p class="mt-2 text-[11.5px] text-ink-faint">Click to toggle the drag-active state.</p>
			</Card>

			<Card padded={false} class="md:col-span-2">
				<div class="px-5 pt-[18px]">
					<div class={eyebrow()}>Table · density-aware</div>
				</div>
				<table class={t.root()}>
					<thead>
						<tr>
							<th class={t.th()}>Name <span class="ml-0.5 text-[11px] text-accent">↓</span></th>
							<th class={t.th({ class: 'w-[100px]' })}>Type</th>
							<th class={t.th({ class: 'w-[110px]' })}>Size</th>
							<th class={t.th({ class: 'w-[130px]' })}>Modified</th>
							<th class={t.th({ class: 'w-[130px]' })}></th>
						</tr>
					</thead>
					<tbody>
						{#each ROWS as r, i}
							<tr class={t.tr({ class: 'group/row' })} data-active={i === 0 ? '1' : '0'}>
								<td class={t.td()}>
									<div class="flex min-w-0 items-center gap-[9px]">
										<div
											class="grid size-[22px] shrink-0 place-items-center rounded-[5px] font-code text-[8px] font-bold text-white"
											style="background: {r.color};"
										>
											{r.ext.toUpperCase()}
										</div>
										<span class="truncate">{r.name}</span>
									</div>
								</td>
								<td class={t.td({ class: cell({ variant: 'type' }) })}>{r.ext}</td>
								<td class={t.td({ class: cell({ variant: 'size' }) })}>{r.size}</td>
								<td class={t.td({ class: cell({ variant: 'date' }) })}>{r.when}</td>
								<td class={t.td({ class: cell({ variant: 'actions' }) })}>
									<div
										class="flex justify-end gap-0.5 opacity-0 transition-opacity duration-100 group-hover/row:opacity-100 group-data-[active=1]/row:opacity-100"
									>
										<IconButton variant="row" size="sm" title="Open">
											<Icon name="Eye" size={14} />
										</IconButton>
										<IconButton variant="row" size="sm" title="Download">
											<Icon name="Download" size={14} />
										</IconButton>
										<IconButton variant="row" size="sm" title="Delete">
											<Icon name="Trash" size={14} />
										</IconButton>
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
				<p class="px-5 pt-3 pb-[18px] text-[11.5px] text-ink-faint">
					Hover a row to reveal its actions. Row height follows the density control above.
				</p>
			</Card>

			<Card>
				<div class={eyebrow()}>Section</div>
				<Section label="Access">
					<Field label="Role" class="mb-0">
						<Select value="user">
							<option value="user">User</option>
						</Select>
					</Field>
				</Section>
			</Card>

			<Card padded={false}>
				<div class="px-5 pt-[18px]">
					<div class={eyebrow()}>Empty state</div>
				</div>
				<EmptyState title="Trash is empty" body="Deleted files appear here for 30 days.">
					{#snippet art()}<Icon name="TrashBin" size={28} />{/snippet}
				</EmptyState>
			</Card>
		</div>
	</div>
</div>
