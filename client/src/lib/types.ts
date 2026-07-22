export type FileCategory = 'img' | 'vid' | 'aud' | 'doc' | 'pdf' | 'zip' | 'code' | 'txt' | 'data' | 'other';

export interface FilerFile {
	id: string;
	name: string;
	ext: string;
	category: FileCategory;
	color: string;
	thumb: string | null;
	size: number;
	modified: string;
	owner: string;
	public: boolean;
	trashed: boolean;
	trashedAt: string | null;
}

export type UserRole = 'admin' | 'user';
export type UserStatus = 'active' | 'suspended';

export interface User {
	id: string;
	name: string;
	username: string;
	role: UserRole;
	avatar?: string | null;
	status?: UserStatus;
	quotaGB?: number;
	usedGB?: number;
	files?: number;
}

export type AuditEventType =
	| 'upload'
	| 'download'
	| 'rename'
	| 'delete'
	| 'restore'
	| 'share'
	| 'login'
	| 'auth_fail'
	| 'user_add'
	| 'user_edit'
	| 'password';

export interface AuditEvent {
	id: string;
	at: string;
	type: AuditEventType;
	actor: string;
	target: string | null;
	ip: string;
	meta: string | null;
}

export type FileSection = 'my' | 'public' | 'trash';
export type ViewMode = 'list' | 'grid';
export type Density = 'compact' | 'cozy' | 'comfy';
export type SortKey = 'name' | 'type' | 'size' | 'modified';
export type SortDir = 'asc' | 'desc';

export interface Toast {
	id: string;
	msg: string;
	onUndo?: () => void;
}
