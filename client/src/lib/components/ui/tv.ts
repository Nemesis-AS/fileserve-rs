import { createTV } from 'tailwind-variants';

/**
 * Shared `tv` instance — import this rather than `tv` from `tailwind-variants`.
 *
 * tailwind-merge only knows Tailwind's stock theme. Our shadow tokens (`shadow-xs`,
 * `shadow-pop`, `shadow-lift`, from the `@theme inline` block in routes/layout.css) are
 * unknown to it, so it parses them as shadow *colors* and lets `shadow-xs shadow-lift`
 * both survive instead of the caller's winning. Registering them as sizes fixes that.
 */
export const tv = createTV({
	twMergeConfig: {
		extend: {
			classGroups: {
				shadow: [{ shadow: ['xs', 'pop', 'lift'] }]
			}
		}
	}
});

export type { VariantProps } from 'tailwind-variants';

/**
 * Svelte's element attribute types allow `class` to be a `ClassValue` (string, array,
 * object, even number). tailwind-merge only accepts strings, so widen-then-narrow:
 * spread the element's attributes but pin `class` back to a plain string.
 */
export type WithClass<T> = Omit<T, 'class'> & { class?: string };
