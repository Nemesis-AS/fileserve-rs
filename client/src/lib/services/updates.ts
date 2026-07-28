import { apiFetch } from './api';

export type InstallPhase = 'idle' | 'downloading' | 'verifying' | 'applying' | 'ready' | 'failed';

export type RestartMode = 'spawn' | 'exit';

export interface Release {
	version: string;
	tagName: string;
	name: string | null;
	body: string | null;
	htmlUrl: string;
	publishedAt: Date | null;
	assetName: string;
	assetSize: number;
}

export interface InstallProgress {
	phase: InstallPhase;
	downloadedBytes: number;
	totalBytes: number;
	version: string | null;
	error: string | null;
}

export interface UpdateStatus {
	currentVersion: string;
	target: string;
	enabled: boolean;
	lastChecked: Date | null;
	checking: boolean;
	checkError: string | null;
	latest: Release | null;
	install: InstallProgress;
}

interface ReleaseDto {
	version: string;
	tag_name: string;
	name: string | null;
	body: string | null;
	html_url: string;
	published_at: string | null;
	asset_name: string;
	asset_size: number;
}

interface InstallDto {
	phase: InstallPhase;
	downloaded_bytes: number;
	total_bytes: number;
	version: string | null;
	error: string | null;
}

interface UpdateStatusDto {
	current_version: string;
	target: string;
	enabled: boolean;
	last_checked: string | null;
	checking: boolean;
	check_error: string | null;
	latest: ReleaseDto | null;
	install: InstallDto;
}

interface VersionDto {
	version: string;
	target: string;
}

interface RestartDto {
	restart_mode: RestartMode;
}

function releaseFromDto(d: ReleaseDto): Release {
	return {
		version: d.version,
		tagName: d.tag_name,
		name: d.name,
		body: d.body,
		htmlUrl: d.html_url,
		publishedAt: d.published_at ? new Date(d.published_at) : null,
		assetName: d.asset_name,
		assetSize: d.asset_size
	};
}

function installFromDto(d: InstallDto): InstallProgress {
	return {
		phase: d.phase,
		downloadedBytes: d.downloaded_bytes,
		totalBytes: d.total_bytes,
		version: d.version,
		error: d.error
	};
}

function fromDto(d: UpdateStatusDto): UpdateStatus {
	return {
		currentVersion: d.current_version,
		target: d.target,
		enabled: d.enabled,
		lastChecked: d.last_checked ? new Date(d.last_checked) : null,
		checking: d.checking,
		checkError: d.check_error,
		latest: d.latest ? releaseFromDto(d.latest) : null,
		install: installFromDto(d.install)
	};
}

export async function getUpdateStatus(): Promise<UpdateStatus> {
	return fromDto(await apiFetch<UpdateStatusDto>('/system/update'));
}

export async function checkForUpdate(): Promise<UpdateStatus> {
	return fromDto(await apiFetch<UpdateStatusDto>('/system/update/check', { method: 'POST' }));
}

/**
 * The version is echoed back so the server can refuse if a newer release landed
 * since this page rendered, rather than installing something unseen.
 */
export async function startInstall(version: string): Promise<InstallProgress> {
	return installFromDto(
		await apiFetch<InstallDto>('/system/update/install', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ version })
		})
	);
}

export async function getInstallProgress(): Promise<InstallProgress> {
	return installFromDto(await apiFetch<InstallDto>('/system/update/progress'));
}

/** Resolves with how the server will restart: itself, or via a supervisor. */
export async function restartServer(): Promise<RestartMode> {
	const { restart_mode } = await apiFetch<RestartDto>('/system/restart', {
		method: 'POST'
	});
	return restart_mode;
}

/**
 * Bypasses `apiFetch` deliberately: that routes a 401 into `handleUnauthorized`,
 * which logs the user out. A proxy answering 502 mid-restart shouldn't sign
 * anyone out. Every failure here just means "not back yet".
 */
export async function pingVersion(): Promise<string | null> {
	try {
		const res = await fetch('/api/v1/system/version', {
			credentials: 'include',
			cache: 'no-store'
		});
		if (!res.ok) return null;
		const body = (await res.json()) as { data?: VersionDto | null };
		return body?.data?.version ?? null;
	} catch {
		return null;
	}
}
