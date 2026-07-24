import { apiFetch } from './api';

const BYTES_PER_GB = 1_073_741_824;

export interface ServerSettings {
	storagePath: string;
	maxUploadGB: number;
	defaultQuotaGB: number;
}

interface SettingsDto {
	storage_path: string;
	tus_max_size: number;
	default_quota_bytes: number;
}

function fromDto(d: SettingsDto): ServerSettings {
	return {
		storagePath: d.storage_path,
		maxUploadGB: d.tus_max_size / BYTES_PER_GB,
		defaultQuotaGB: d.default_quota_bytes / BYTES_PER_GB
	};
}

export async function getSettings(): Promise<ServerSettings> {
	return fromDto(await apiFetch<SettingsDto>('/settings'));
}

export async function updateSettings(updates: Partial<ServerSettings>): Promise<ServerSettings> {
	const patch: Partial<SettingsDto> = {};
	if (updates.storagePath !== undefined) patch.storage_path = updates.storagePath;
	if (updates.maxUploadGB !== undefined)
		patch.tus_max_size = Math.round(updates.maxUploadGB * BYTES_PER_GB);
	if (updates.defaultQuotaGB !== undefined)
		patch.default_quota_bytes = Math.round(updates.defaultQuotaGB * BYTES_PER_GB);

	return fromDto(
		await apiFetch<SettingsDto>('/settings', {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(patch)
		})
	);
}
