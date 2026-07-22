<script lang="ts">
	// The fileserve.rs mark: a dark squircle tile holding three stacked slots —
	// a server rack and a file list read at once — with an accent status node
	// that mirrors the dot in the ".rs" wordmark. Self-contained; theme-independent.
	let { size = 24 }: { size?: number } = $props();
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 100 100"
	class="shrink-0"
	role="img"
	aria-label="fileserve.rs"
>
	<defs>
		<linearGradient id="fs-node" x1="0" y1="0" x2="1" y2="1">
			<stop offset="0" stop-color="#5b93ff" />
			<stop offset="1" stop-color="#2563eb" />
		</linearGradient>
		<linearGradient id="fs-sheen" x1="0" y1="0" x2="0" y2="1">
			<stop offset="0" stop-color="#ffffff" stop-opacity=".08" />
			<stop offset=".5" stop-color="#ffffff" stop-opacity="0" />
		</linearGradient>
	</defs>
	<rect x="4" y="4" width="92" height="92" rx="24" fill="#18181b" />
	<rect x="4" y="4" width="92" height="92" rx="24" fill="url(#fs-sheen)" />
	<rect x="28" y="30" width="30" height="8.5" rx="4.25" fill="#fafafa" />
	<circle cx="67.5" cy="34.25" r="5.4" fill="url(#fs-node)" />
	<rect x="28" y="45.75" width="44" height="8.5" rx="4.25" fill="#fafafa" opacity=".92" />
	<rect x="28" y="61.5" width="36" height="8.5" rx="4.25" fill="#fafafa" opacity=".6" />
</svg>
