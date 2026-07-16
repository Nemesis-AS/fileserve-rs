import type { Snippet } from 'svelte';
import { tv, type VariantProps } from '../tv.js';

/**
 * Port of `.scrim` + `.modal*`. All four modals (Delete/Rename/Upload/Properties)
 * hand-rolled the same scrim, header, close button, footer and transitions; this owns
 * all of it.
 *
 * Deliberately NOT handling Escape: no modal closes on Escape today, and adding it here
 * would be a behaviour change rather than a refactor. Worth doing later, on purpose.
 */
export const modal = tv({
	slots: {
		scrim: 'fixed inset-0 z-100 grid place-items-center bg-overlay p-5',
		root: 'flex max-h-[calc(100vh-40px)] w-[480px] max-w-full flex-col overflow-hidden rounded-xl border border-edge bg-surface shadow-lift',
		header: 'flex shrink-0 items-center gap-2 px-[18px] pt-4 pb-3',
		title: 'm-0 text-[15px] font-semibold tracking-[-0.005em]',
		body: 'min-h-0 flex-1 overflow-auto px-[18px] pt-1 pb-[18px]',
		footer: 'flex shrink-0 gap-2 border-t border-edge bg-elevated px-[18px] py-3'
	},
	variants: {
		size: {
			default: {},
			wide: { root: 'w-[580px]' }
		},
		/** `.modal__ft--end` right-aligns the footer actions. */
		footerAlign: {
			start: {},
			end: { footer: 'justify-end' }
		}
	},
	defaultVariants: { size: 'default', footerAlign: 'end' }
});

export type ModalSize = VariantProps<typeof modal>['size'];

export interface ModalProps {
	title: string;
	size?: ModalSize;
	footerAlign?: 'start' | 'end';
	onClose: () => void;
	/**
	 * When set, the modal root renders as a <form> and this handles submit — which is
	 * how RenameModal gets Enter-to-submit. Otherwise the root is a plain <div>.
	 */
	onsubmit?: (e: SubmitEvent) => void;
	/** Extra controls rendered in the header, between the title and the close button. */
	headerExtra?: Snippet;
	footer?: Snippet;
	class?: string;
	children: Snippet;
}

export { default as Modal } from './Modal.svelte';
