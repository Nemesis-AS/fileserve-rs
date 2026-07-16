import type { HTMLButtonAttributes } from 'svelte/elements';
import type { Snippet } from 'svelte';
import { tv, type VariantProps, type WithClass } from '../tv.js';

/**
 * One component for what layout.css spelled five ways:
 *   .icon-btn         30x30, icon 16 — topbar
 *   .row-action       26x26, icon 14 — table row actions (black/white wash hover)
 *   .modal__close     26x26, icon 14 — modal close (row-hover wash)
 *   .gridtile__action 26x26, icon 14 — overlay on thumbnails (dark scrim)
 *   .upload-row__x    22x22, icon 14 — upload queue remove
 *
 * `variant` picks the hover treatment, `size` the box. The three that shared a hover
 * treatment but differed in size collapse into size variants of `default`.
 */
export const iconButton = tv({
	base: 'grid shrink-0 cursor-pointer place-items-center border-0 bg-transparent transition-colors duration-100',
	variants: {
		variant: {
			// .icon-btn / .modal__close / .upload-row__x — tint wash from the row-hover token
			default: 'text-ink-muted hover:bg-row-hover hover:text-ink',
			// .row-action — a neutral wash that flips polarity in dark mode
			row: 'text-ink-muted hover:bg-black/[.06] hover:text-ink dark:hover:bg-white/[.08]',
			// .gridtile__action — sits on top of imagery, so it carries its own scrim
			overlay: 'bg-black/55 text-white backdrop-blur-[8px] hover:bg-black/75'
		},
		size: {
			xs: 'size-[22px] rounded-[5px] [&_svg]:size-[14px]',
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
