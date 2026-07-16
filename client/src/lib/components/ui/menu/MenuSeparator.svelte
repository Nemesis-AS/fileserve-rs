<script lang="ts">
	/** Port of `.menu__sep`. */
	let { class: klass = '' }: { class?: string } = $props();
</script>

<div class="my-1 h-px bg-edge {klass}"></div>
