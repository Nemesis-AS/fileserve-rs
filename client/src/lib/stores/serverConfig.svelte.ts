import { getServerConfig } from '$lib/services/config';
import type { ServerConfig } from '$lib/types';

/**
 * Defaults chosen so a failed fetch degrades to "an ordinary self-hosted
 * server with no client-side upload limit". Getting this wrong the other way
 * would block uploads on a server that was fine.
 */
const FALLBACK: ServerConfig = { demo: false, maxUploadBytes: Number.POSITIVE_INFINITY };

let _config = $state<ServerConfig>(FALLBACK);
let _loaded = $state(false);

export const serverConfig = {
	get config() {
		return _config;
	},

	get loaded() {
		return _loaded;
	},

	get isDemo() {
		return _config.demo;
	},

	async load() {
		try {
			_config = await getServerConfig();
		} catch {
			_config = FALLBACK;
		} finally {
			_loaded = true;
		}
	}
};
