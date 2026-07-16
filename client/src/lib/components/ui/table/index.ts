import { tv, type VariantProps } from '../tv.js';

/**
 * Two tables existed: `.table` (the file list — sticky header, density-driven row
 * height, borderless) and `.users` (the admin list — boxed, fixed padding). They are
 * different enough to keep as one `variant` rather than forcing them together.
 *
 * `h-row` / `px-cell` resolve through `--row-h` / `--gap-cell`, so the file table keeps
 * responding to [data-density] with no JS.
 */
export const table = tv({
	slots: {
		root: 'w-full border-collapse text-[13px]',
		th: 'text-left font-medium whitespace-nowrap text-ink-muted',
		td: 'align-middle',
		tr: ''
	},
	variants: {
		variant: {
			files: {
				th: 'sticky top-0 z-2 h-[34px] cursor-pointer border-b border-edge bg-surface px-cell text-[12px] select-none hover:text-ink',
				td: 'h-row border-b border-edge px-cell whitespace-nowrap',
				tr: 'cursor-default transition-colors duration-[80ms] hover:bg-row-hover data-[active=1]:bg-row-active last:[&>td]:border-b-0'
			},
			users: {
				root: 'overflow-hidden rounded-[10px] border border-edge bg-surface',
				th: 'border-b border-edge bg-elevated px-[14px] py-2.5 text-[12px]',
				td: 'border-b border-edge px-[14px] py-3',
				tr: 'hover:bg-row-hover last:[&>td]:border-b-0'
			}
		}
	},
	defaultVariants: { variant: 'files' }
});

/**
 * Port of the `.cell-*` classes. No shared base on purpose: `.cell-actions` was never
 * part of the muted/tabular group in layout.css, so hoisting those up would tint it.
 */
export const cell = tv({
	variants: {
		variant: {
			default: '',
			size: 'text-[12.5px] text-ink-muted tabular-nums',
			date: 'text-[12.5px] text-ink-muted tabular-nums',
			owner: 'text-[12.5px] text-ink-muted tabular-nums',
			type: 'font-code text-[11.5px] text-ink-muted uppercase tabular-nums',
			actions: 'w-[1%] text-right whitespace-nowrap'
		}
	},
	defaultVariants: { variant: 'default' }
});

export type TableVariant = VariantProps<typeof table>['variant'];
export type CellVariant = VariantProps<typeof cell>['variant'];
