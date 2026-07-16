import type { HTMLInputAttributes, HTMLSelectAttributes } from 'svelte/elements';
import type { Snippet } from 'svelte';
import { tv, type VariantProps, type WithClass } from '../tv.js';

/**
 * Port of `.input` / `.input--lg`.
 *
 * `sm` is the audit filter select (`.audit-filters__select`). It is a real size rather
 * than an override inside the filter variant: when `Select` extends this recipe, tv
 * applies the parent's `size` AFTER the child's `variant`, so a height set in the child
 * would silently lose to the `md` default.
 */
export const input = tv({
	base: 'w-full rounded-[7px] border border-edge-strong bg-surface px-[10px] font-system text-ink outline-0 transition-[border-color,box-shadow] duration-150 focus:border-accent focus:shadow-[0_0_0_3px_var(--accent-soft)] disabled:cursor-not-allowed disabled:opacity-50',
	variants: {
		size: {
			sm: 'h-[30px] text-[12.5px]',
			md: 'h-[34px] text-[13.5px]',
			lg: 'h-[38px] text-[14px]'
		}
	},
	defaultVariants: { size: 'md' }
});

export type InputSize = VariantProps<typeof input>['size'];

export interface InputProps extends Omit<WithClass<HTMLInputAttributes>, 'size'> {
	size?: InputSize;
	/** number covers `type="number"` bindings, which Svelte coerces for us. */
	value?: string | number;
	/** `bind:ref` to reach the underlying element (focus, selection). */
	ref?: HTMLInputElement | null;
}

export interface SelectProps extends Omit<WithClass<HTMLSelectAttributes>, 'size'> {
	size?: InputSize;
	value?: string | number;
	children: Snippet;
}

export { default as Input } from './Input.svelte';
export { default as Select } from './Select.svelte';
