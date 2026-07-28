import type { AuditEvent } from '$lib/types';

const NOW = Date.now();
const AT = (mins: number) => new Date(NOW - mins * 60_000).toISOString();

export const MOCK_AUDIT: AuditEvent[] = [
	{
		id: 'e60',
		at: AT(2),
		type: 'upload',
		actor: 'alex',
		target: 'reading-notes.md',
		ip: '10.0.1.42',
		meta: '22 KB'
	},
	{
		id: 'e59',
		at: AT(8),
		type: 'download',
		actor: 'sam',
		target: 'mountain-lake.jpg',
		ip: '10.0.1.43',
		meta: 'shared link'
	},
	{
		id: 'e58',
		at: AT(14),
		type: 'login',
		actor: 'alex',
		target: null,
		ip: '10.0.1.42',
		meta: 'web'
	},
	{
		id: 'e57',
		at: AT(33),
		type: 'rename',
		actor: 'alex',
		target: 'travel-itinerary.docx',
		ip: '10.0.1.42',
		meta: 'from "trip-may.docx"'
	},
	{
		id: 'e56',
		at: AT(40),
		type: 'share',
		actor: 'alex',
		target: 'workspace.jpg',
		ip: '10.0.1.42',
		meta: 'made public'
	},
	{
		id: 'e55',
		at: AT(58),
		type: 'delete',
		actor: 'alex',
		target: 'draft-letter.md',
		ip: '10.0.1.42',
		meta: 'to Trash'
	},
	{
		id: 'e54',
		at: AT(74),
		type: 'upload',
		actor: 'leo',
		target: 'shared-album-link.txt',
		ip: '10.0.1.44',
		meta: '180 B'
	},
	{
		id: 'e53',
		at: AT(91),
		type: 'login',
		actor: 'leo',
		target: null,
		ip: '10.0.1.44',
		meta: 'web'
	},
	{
		id: 'e52',
		at: AT(120),
		type: 'download',
		actor: 'alex',
		target: 'home-server.tf',
		ip: '10.0.1.42',
		meta: null
	},
	{
		id: 'e51',
		at: AT(180),
		type: 'upload',
		actor: 'alex',
		target: 'birthday-2025.mp4',
		ip: '10.0.1.42',
		meta: '412 MB'
	},
	{
		id: 'e50',
		at: AT(245),
		type: 'auth_fail',
		actor: 'jules',
		target: null,
		ip: '185.243.218.50',
		meta: 'wrong password'
	},
	{
		id: 'e49',
		at: AT(248),
		type: 'auth_fail',
		actor: 'jules',
		target: null,
		ip: '185.243.218.50',
		meta: 'wrong password'
	},
	{
		id: 'e48',
		at: AT(252),
		type: 'auth_fail',
		actor: 'jules',
		target: null,
		ip: '185.243.218.50',
		meta: 'wrong password'
	},
	{
		id: 'e47',
		at: AT(290),
		type: 'user_edit',
		actor: 'alex',
		target: '@jules',
		ip: '10.0.1.42',
		meta: 'suspended'
	},
	{
		id: 'e46',
		at: AT(310),
		type: 'share',
		actor: 'alex',
		target: 'movie-night-poster.png',
		ip: '10.0.1.42',
		meta: 'made public'
	},
	{
		id: 'e45',
		at: AT(340),
		type: 'download',
		actor: 'mira',
		target: 'recipe-shakshuka.md',
		ip: '10.0.1.45',
		meta: 'shared link'
	},
	{
		id: 'e44',
		at: AT(420),
		type: 'login',
		actor: 'sam',
		target: null,
		ip: '10.0.1.43',
		meta: 'web'
	},
	{
		id: 'e43',
		at: AT(540),
		type: 'restore',
		actor: 'alex',
		target: 'old-notes.txt',
		ip: '10.0.1.42',
		meta: 'from Trash'
	},
	{
		id: 'e42',
		at: AT(720),
		type: 'user_add',
		actor: 'alex',
		target: '@mira',
		ip: '10.0.1.42',
		meta: 'role: user'
	},
	{
		id: 'e41',
		at: AT(840),
		type: 'download',
		actor: 'alex',
		target: 'family-photos-2024.zip',
		ip: '10.0.1.42',
		meta: null
	},
	{
		id: 'e40',
		at: AT(1020),
		type: 'login',
		actor: 'alex',
		target: null,
		ip: '10.0.1.42',
		meta: 'web'
	},
	{
		id: 'e39',
		at: AT(1260),
		type: 'upload',
		actor: 'alex',
		target: 'household-budget.xlsx',
		ip: '10.0.1.42',
		meta: '402 KB'
	},
	{
		id: 'e38',
		at: AT(1380),
		type: 'delete',
		actor: 'leo',
		target: 'temp-export.csv',
		ip: '10.0.1.44',
		meta: 'to Trash'
	},
	{
		id: 'e37',
		at: AT(1620),
		type: 'password',
		actor: 'sam',
		target: null,
		ip: '10.0.1.43',
		meta: 'changed by user'
	},
	{
		id: 'e36',
		at: AT(2200),
		type: 'login',
		actor: 'mira',
		target: null,
		ip: '10.0.1.45',
		meta: 'web'
	},
	{
		id: 'e35',
		at: AT(2880),
		type: 'share',
		actor: 'leo',
		target: 'shared-album-link.txt',
		ip: '10.0.1.44',
		meta: 'made public'
	}
];
