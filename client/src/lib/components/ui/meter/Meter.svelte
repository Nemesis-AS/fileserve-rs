<script lang="ts">
	import { meter, type MeterProps } from './index.js';

	let { value, size, radius, speed, color, class: klass }: MeterProps = $props();

	const { track, bar } = $derived(meter({ size, radius, speed }));
	const pct = $derived(Math.max(0, Math.min(100, value)));
</script>

<div class={track({ class: klass })}>
	<i class={bar()} style="width: {pct}%; background: {color ?? 'var(--accent)'};"></i>
</div>
