import type { HTMLButtonAttributes } from 'svelte/elements';
import type { Snippet } from 'svelte';
import { tv, type VariantProps, type WithClass } from '../tv.js';

/**
 * Port of `.btn` / `.btn--ghost` / `.btn--danger` / `.btn--lg`.
 *
 * `transition-opacity` lives on the base because the original `.btn--ghost` never
 * overrode `transition`, so its background hover was instant. Keep it that way.
 * The `[&_svg]:size-[14px]` reproduces `.btn svg { width:14px; height:14px }`,
 * which forced icon size regardless of the icon's own size prop.
 */
export const button = tv({
	base: 'inline-flex shrink-0 cursor-pointer items-center gap-1.5 rounded-[7px] border-0 font-medium whitespace-nowrap transition-opacity duration-100 [&_svg]:size-[14px]',
	variants: {
		variant: {
			solid: 'bg-ink text-surface hover:opacity-[.88]',
			ghost:
				'border border-edge-strong bg-transparent text-ink hover:bg-row-hover hover:opacity-100',
			danger: 'bg-danger text-white hover:opacity-90'
		},
		size: {
			sm: 'h-[30px] px-3 text-[13px]',
			lg: 'h-9 px-4 text-[14px]'
		}
	},
	defaultVariants: { variant: 'solid', size: 'sm' }
});

export type ButtonVariant = VariantProps<typeof button>['variant'];
export type ButtonSize = VariantProps<typeof button>['size'];

export interface ButtonProps extends WithClass<HTMLButtonAttributes> {
	variant?: ButtonVariant;
	size?: ButtonSize;
	children: Snippet;
}

export { default as Button } from './Button.svelte';
