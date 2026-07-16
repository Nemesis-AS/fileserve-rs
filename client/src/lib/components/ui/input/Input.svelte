<script lang="ts">
	import { input, type InputProps } from './index.js';

	let {
		size,
		class: klass,
		value = $bindable(),
		ref = $bindable(),
		...rest
	}: InputProps = $props();
</script>

<input bind:this={ref} bind:value class={input({ size, class: klass })} {...rest} />
