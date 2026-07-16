<script lang="ts">
	type IconName =
		| 'Search' | 'Upload' | 'Download' | 'Eye' | 'Trash' | 'Pencil' | 'Info'
		| 'More' | 'Close' | 'ChevronR' | 'ChevronL' | 'ChevronD' | 'ArrowDown' | 'ArrowUp'
		| 'Files' | 'Public' | 'TrashBin' | 'Users' | 'Settings' | 'LogOut'
		| 'Moon' | 'Sun' | 'Plus' | 'Lock' | 'Link' | 'Folder' | 'Copy' | 'Check'
		| 'Refresh';

	let { name, size = 16, class: cls = '', style = '' }: { name: IconName; size?: number; class?: string; style?: string } = $props();

	const PATHS: Record<IconName, string | string[]> = {
		Search:    ['M21 21l-4.3-4.3', 'M10.5 18a7.5 7.5 0 1 0 0-15 7.5 7.5 0 0 0 0 15z'],
		Upload:    ['M12 16V4', 'M6 10l6-6 6 6', 'M4 20h16'],
		Download:  ['M12 4v12', 'M6 12l6 6 6-6', 'M4 20h16'],
		Eye:       ['M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z', 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z'],
		Trash:     ['M3 6h18', 'M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2', 'M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6', 'M10 11v6', 'M14 11v6'],
		Pencil:    ['M12 20h9', 'M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4 12.5-12.5z'],
		Info:      ['M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z', 'M12 11v6', 'M12 7.5h.01'],
		More:      ['M5 12h.01', 'M12 12h.01', 'M19 12h.01'],
		Close:     ['M18 6 6 18', 'm6 6 12 12'],
		ChevronR:  'm9 6 6 6-6 6',
		ChevronL:  'm15 6-6 6 6 6',
		ChevronD:  'm6 9 6 6 6-6',
		ArrowDown: ['M12 5v14', 'm19 12-7 7-7-7'],
		ArrowUp:   ['M12 19V5', 'm5 12 7-7 7 7'],
		Files:     ['M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z', 'M14 2v6h6'],
		Public:    ['M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z', 'M3 12h18', 'M12 3a14 14 0 0 1 0 18', 'M12 3a14 14 0 0 0 0 18'],
		TrashBin:  ['M3 6h18', 'M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2', 'M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6'],
		Users:     ['M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2', 'M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z', 'M23 21v-2a4 4 0 0 0-3-3.87', 'M16 3.13a4 4 0 0 1 0 7.75'],
		Settings:  ['M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h0a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h0a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v0a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z', 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z'],
		LogOut:    ['M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4', 'm16 17 5-5-5-5', 'M21 12H9'],
		Moon:      'M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z',
		Sun:       ['M12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10z', 'M12 1v2', 'M12 21v2', 'M4.22 4.22l1.42 1.42', 'M18.36 18.36l1.42 1.42', 'M1 12h2', 'M21 12h2', 'M4.22 19.78l1.42-1.42', 'M18.36 5.64l1.42-1.42'],
		Plus:      ['M12 5v14', 'M5 12h14'],
		Lock:      ['M5 11h14v10H5z', 'M8 11V7a4 4 0 1 1 8 0v4'],
		Link:      ['M10 13a5 5 0 0 0 7.07 0l3-3a5 5 0 0 0-7.07-7.07l-1.5 1.5', 'M14 11a5 5 0 0 0-7.07 0l-3 3a5 5 0 0 0 7.07 7.07l1.5-1.5'],
		Folder:    'M3 7a2 2 0 0 1 2-2h4l2 3h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z',
		Copy:      ['M9 9h11a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H9a2 2 0 0 1-2-2V11a2 2 0 0 1 2-2z', 'M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1'],
		Check:     'M20 6 9 17l-5-5',
		Refresh:   ['M23 4v6h-6', 'M1 20v-6h6', 'M3.51 9a9 9 0 0 1 14.85-3.36L23 10', 'M1 14l4.64 4.36A9 9 0 0 0 20.49 15'],
	};

	const ds = PATHS[name];
	const paths = Array.isArray(ds) ? ds : [ds];
</script>

<svg
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="1.6"
	stroke-linecap="round"
	stroke-linejoin="round"
	width={size}
	height={size}
	class={cls}
	{style}
	aria-hidden="true"
>
	{#each paths as d}
		<path {d} />
	{/each}
</svg>
