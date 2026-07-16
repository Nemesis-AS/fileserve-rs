<script lang="ts">
	import { iconButton, type IconButtonProps } from './index.js';

	let {
		variant,
		size,
		class: klass,
		type = 'button',
		children,
		...rest
	}: IconButtonProps = $props();
</script>

<button {type} class={iconButton({ variant, size, class: klass })} {...rest}>
	{@render children()}
</button>
