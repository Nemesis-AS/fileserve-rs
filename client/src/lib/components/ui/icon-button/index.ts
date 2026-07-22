import type { HTMLButtonAttributes } from 'svelte/elements';
import type { Snippet } from 'svelte';
import { tv, type VariantProps, type WithClass } from '../tv.js';

export const iconButton = tv({
	base: 'grid shrink-0 cursor-pointer place-items-center border-0 bg-transparent transition-colors duration-100',
	variants: {
		variant: {
			default: 'text-ink-muted hover:bg-row-hover hover:text-ink',
			row: 'text-ink-muted hover:bg-black/[.06] hover:text-ink dark:hover:bg-white/[.08]',
			overlay: 'bg-black/55 text-white backdrop-blur-[8px] hover:bg-black/75'
		},
		size: {
			xs: 'size-[22px] rounded-[5px] [&_svg]:size-3',
			sm: 'size-[26px] rounded-[6px] [&_svg]:size-[14px]',
			md: 'size-[30px] rounded-[7px] [&_svg]:size-4'
		}
	},
	defaultVariants: { variant: 'default', size: 'md' }
});

export type IconButtonVariant = VariantProps<typeof iconButton>['variant'];
export type IconButtonSize = VariantProps<typeof iconButton>['size'];

export interface IconButtonProps extends WithClass<HTMLButtonAttributes> {
	variant?: IconButtonVariant;
	size?: IconButtonSize;
	children: Snippet;
}

export { default as IconButton } from './IconButton.svelte';
