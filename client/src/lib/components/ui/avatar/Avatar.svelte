<script lang="ts">
	import { avatar, type AvatarProps } from './index.js';
	import { initials } from '$lib/utils/file';

	let { name, size, class: klass, ...rest }: AvatarProps = $props();
</script>

<div class={avatar({ size, class: klass })} {...rest}>{initials(name)}</div>
