<script lang="ts">
	import { button, type ButtonProps } from './index.js';

	let {
		variant,
		size,
		class: klass,
		type = 'button',
		children,
		...rest
	}: ButtonProps = $props();
</script>

<button {type} class={button({ variant, size, class: klass })} {...rest}>
	{@render children()}
</button>
