<script lang="ts">
	import { menuItem, type MenuItemProps } from './index.js';

	let { danger, class: klass, type = 'button', children, ...rest }: MenuItemProps = $props();
</script>

<button {type} class={menuItem({ danger, class: klass })} {...rest}>
	{@render children()}
</button>
