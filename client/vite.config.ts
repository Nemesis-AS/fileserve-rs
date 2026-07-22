import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import Icons from 'unplugin-icons/vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit(),
		Icons({
			compiler: 'svelte'
		})
	],
	server: {
		proxy: {
			// Proxy API calls to the Rust backend so requests stay same-origin and
			// the auth_token cookie is set/sent without CORS credential rules.
			'/api': {
				target: 'http://localhost:8112',
				changeOrigin: true
			}
		}
	}
});
