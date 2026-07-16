<script lang="ts">
	import type { Snippet } from 'svelte';
	import { tv } from '../tv.js';

	/** Port of `.field-hint`. The negative margin claws back the parent's 6px gap. */
	const hint = tv({ base: '-mt-0.5 text-[11.5px] text-ink-muted' });

	let { class: klass, children }: { class?: string; children: Snippet } = $props();
</script>

<div class={hint({ class: klass })}>{@render children()}</div>
