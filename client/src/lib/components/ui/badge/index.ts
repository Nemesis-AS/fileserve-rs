import type { Snippet } from 'svelte';
import type { HTMLAttributes } from 'svelte/elements';
import { tv, type VariantProps, type WithClass } from '../tv.js';

/**
 * Port of `.badge` + its `[data-tone]` cases. `active`/`suspended` also render a
 * `currentColor` dot (the old `.badge[data-tone] i`), which Badge draws itself.
 */
export const badge = tv({
	base: 'inline-flex items-center gap-1 rounded-full border border-edge bg-surface px-[7px] py-px text-[11px] font-medium whitespace-nowrap text-ink-muted',
	variants: {
		tone: {
			neutral: '',
			admin: 'border-transparent bg-accent-soft text-accent-ink',
			active: 'text-ok',
			suspended: 'text-warn'
		}
	},
	defaultVariants: { tone: 'neutral' }
});

export type BadgeTone = NonNullable<VariantProps<typeof badge>['tone']>;

export interface BadgeProps extends WithClass<HTMLAttributes<HTMLSpanElement>> {
	tone?: BadgeTone;
	children: Snippet;
}

export { default as Badge } from './Badge.svelte';
