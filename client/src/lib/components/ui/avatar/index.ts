import type { HTMLAttributes } from 'svelte/elements';
import { tv, type VariantProps, type WithClass } from '../tv.js';

export const avatar = tv({
	base: 'grid shrink-0 place-items-center rounded-full bg-gradient-to-br from-[#6691ff] to-[#b87cff] font-semibold tracking-[0.02em] text-white',
	variants: {
		size: {
			sm: 'size-[26px] text-[11px]',
			md: 'size-[28px] text-[12px]',
			lg: 'size-12 text-[18px]'
		}
	},
	defaultVariants: { size: 'sm' }
});

export type AvatarSize = VariantProps<typeof avatar>['size'];

export interface AvatarProps extends WithClass<HTMLAttributes<HTMLDivElement>> {
	/** Full name — initials are derived from it. */
	name: string;
	size?: AvatarSize;
}

export { default as Avatar } from './Avatar.svelte';
