import type { Snippet } from 'svelte';

export interface FieldProps {
	/** Rendered as the <label> above the control. */
	label?: string;
	/** Small muted text below the control (`.field-hint`). */
	hint?: string;
	/** Set when the label should point at a control by id. */
	for?: string;
	class?: string;
	children: Snippet;
}

export { default as Field } from './Field.svelte';
export { default as Hint } from './Hint.svelte';
