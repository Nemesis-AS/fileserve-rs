<script>
  import { resolve } from "$app/paths";

  const filesLink = resolve("/files");
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://svelte.dev/docs/kit">svelte.dev/docs/kit</a> to read the documentation</p>
<a href={filesLink} class="text-blue-500">Go to files</a>
