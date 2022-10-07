<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { LogicalPosition } from '@tauri-apps/api/window';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { AnnotationGroup } from '../../../../src-tauri/bindings/features/code_annotations/AnnotationGroup';
	import type { AnnotationShape } from '../../../../src-tauri/bindings/features/code_annotations/AnnotationShape';
	import type { AnnotationEvent } from '../../../../src-tauri/bindings/features/node_annotation/AnnotationEvent';
	import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
	import { colors } from '../../../themes';
	import { BORDER_WIDTH, compute_bracket_highlight_lines } from './bracket_highlight';

	export let code_document_rect: LogicalFrame;
	export let annotation_section: LogicalFrame;
	export let active_window_uid: number;

	let annotation_group: AnnotationGroup | undefined;

	let top_rectangle: LogicalFrame | null = null;
	let bottom_rectangle: LogicalFrame | null = null;

	let opening_bracket_box: LogicalFrame | null = null;
	let closing_bracket_box: LogicalFrame | null = null;

	const listen_to_node_annotation_events = async () => {
		let node_annotation_channel: ChannelList = 'NodeAnnotationEvent';
		await listen(node_annotation_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(event.payload as string) as AnnotationEvent;
			switch (event_type) {
				case 'AddAnnotationGroup':
					let added_group = payload as AnnotationGroup;

					if (added_group.feature === 'BracketHighlight') {
						annotation_group = added_group;

						update_closing_bracket();
						update_opening_bracket();
						update_rectangles();
					}

					break;
				case 'UpdateAnnotationGroup':
					let updated_group = payload as AnnotationGroup;

					if (updated_group.feature === 'BracketHighlight') {
						annotation_group = updated_group;

						update_closing_bracket();
						update_opening_bracket();
						update_rectangles();
					}

					break;
				case 'RemoveAnnotationGroup':
					let group_id = payload as string;

					if (annotation_group?.id === group_id) {
						annotation_group = undefined;
						opening_bracket_box = null;
						closing_bracket_box = null;
						top_rectangle = null;
						bottom_rectangle = null;
					}

					break;
				default:
					break;
			}
		});
	};
	listen_to_node_annotation_events();

	const update_opening_bracket = () => {
		opening_bracket_box = null;

		if (annotation_group == null) {
			return;
		}

		let bracket_open = annotation_group.annotations.find(
			(annotation) => annotation.kind === 'OpeningBracket'
		);

		if (!bracket_open) {
			return;
		}

		if (bracket_open.shapes[0] === undefined) {
			return;
		}

		if (!is_rectangle(bracket_open.shapes[0])) {
			return;
		}

		opening_bracket_box = bracket_open.shapes[0].Rectangle;
	};

	const update_closing_bracket = () => {
		closing_bracket_box = null;

		if (annotation_group == null) {
			return;
		}

		let bracket_close = annotation_group.annotations.find(
			(annotation) => annotation.kind === 'ClosingBracket'
		);

		if (!bracket_close) {
			return;
		}

		if (bracket_close.shapes[0] === undefined) {
			return;
		}

		if (!is_rectangle(bracket_close.shapes[0])) {
			return;
		}

		closing_bracket_box = bracket_close.shapes[0].Rectangle;
	};

	const update_rectangles = async () => {
		top_rectangle = null;
		bottom_rectangle = null;

		let lines_start_pos = null;
		let lines_end_pos = null;
		let elbow_pos = null;

		if (annotation_group == null) {
			return;
		}

		let lines_start = annotation_group.annotations.find(
			(annotation) => annotation.kind === 'LineStart'
		);

		let lines_end = annotation_group.annotations.find(
			(annotation) => annotation.kind === 'LineEnd'
		);

		let elbow = annotation_group.annotations.find((annotation) => annotation.kind === 'Elbow');

		if (lines_start && lines_start.shapes[0] !== undefined) {
			if (is_point(lines_start.shapes[0])) {
				lines_start_pos = lines_start.shapes[0].Point;
			}
		}

		if (lines_end && lines_end.shapes[0] !== undefined) {
			if (is_point(lines_end.shapes[0])) {
				lines_end_pos = lines_end.shapes[0].Point;
			}
		}

		if (elbow) {
			if (elbow.shapes[0] !== undefined) {
				if (is_point(elbow.shapes[0])) {
					elbow_pos = elbow.shapes[0].Point;
				}
			} else {
				elbow_pos = {
					x:
						annotation_section.origin.x -
						code_document_rect.origin.x +
						annotation_section.size.width,
					y: 0
				};
			}
		}

		const rectangles = compute_bracket_highlight_lines(
			lines_start_pos,
			lines_end_pos,
			elbow_pos,
			code_document_rect.size.height
		);

		top_rectangle = rectangles.top_rect;
		bottom_rectangle = rectangles.bottom_rect;
	};

	function is_rectangle(shape: AnnotationShape): shape is { Rectangle: LogicalFrame } {
		return shape.hasOwnProperty('Rectangle');
	}

	function is_point(shape: AnnotationShape): shape is { Point: LogicalPosition } {
		return shape.hasOwnProperty('Point');
	}

	const round_value = (value: number, precision: number): number => {
		const factor = Math.pow(10, precision || 0);
		return Math.round(value * factor) / factor;
	};
</script>

{#if annotation_group && annotation_group.editor_window_uid == active_window_uid}
	{#if opening_bracket_box}
		<div
			style="position: absolute; 
			top: {round_value(opening_bracket_box.origin.y, 2)}px; 
			left: {round_value(opening_bracket_box.origin.x, 2)}px; 
			width: {round_value(opening_bracket_box.size.width, 2)}px;
			height: {round_value(opening_bracket_box.size.height, 2)}px; 
			border-style: solid; border-width: {BORDER_WIDTH}px; border-color: {colors.inactive};"
		/>
	{/if}
	{#if closing_bracket_box !== null && code_document_rect !== null}
		<div
			style="position: absolute; 
			top: {round_value(closing_bracket_box.origin.y, 2)}px; 
			left: {round_value(closing_bracket_box.origin.x, 2)}px; 
			width: {round_value(closing_bracket_box.size.width, 2)}px;
			height: {round_value(closing_bracket_box.size.height, 2)}px; 
			border-style: solid; 
			border-width: {BORDER_WIDTH}px; 
			border-color: {colors.inactive};"
		/>
	{/if}
	{#if top_rectangle !== null}
		<div
			style="position: absolute; 
			top: {round_value(top_rectangle.origin.y, 2)}px; 
			left: {round_value(top_rectangle.origin.x, 2)}px; 
			width: {round_value(top_rectangle.size.width, 2)}px;
			height: {round_value(top_rectangle.size.height, 2)}px; 
			border-style: solid; 
			border-color: {colors.inactive}; 
			border-top-width: {BORDER_WIDTH}px; 
			border-left-width: {BORDER_WIDTH}px;
			border-right-width: 0; 
			border-bottom-width: 0;"
		/>
	{/if}
	{#if bottom_rectangle !== null}
		<div
			style="position: absolute; 
			top: {round_value(bottom_rectangle.origin.y, 2)}px; 
			left: {round_value(bottom_rectangle.origin.x, 2)}px; 
			width: {round_value(bottom_rectangle.size.width, 2)}px;
			height: {round_value(bottom_rectangle.size.height, 2)}px; 
			border-bottom-style: solid; 
			border-left-width: {BORDER_WIDTH}px;
			border-bottom-width: {BORDER_WIDTH}px; 
			border-color: {colors.inactive};"
		/>
	{/if}
{/if}
