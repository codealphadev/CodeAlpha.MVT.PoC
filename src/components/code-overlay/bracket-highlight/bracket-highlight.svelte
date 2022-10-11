<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { AnnotationGroup } from '../../../../src-tauri/bindings/features/code_annotations/AnnotationGroup';
	import type { AnnotationShape } from '../../../../src-tauri/bindings/features/code_annotations/AnnotationShape';
	import type { AnnotationEvent } from '../../../../src-tauri/bindings/features/node_annotation/AnnotationEvent';
	import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
	import type { LogicalPosition } from '../../../../src-tauri/bindings/geometry/LogicalPosition';

	import { colors } from '../../../themes';
	import { is_point, try_get_kind_as_rectangle } from '../annotation_utils';
	import { BORDER_WIDTH, compute_bracket_highlight_lines } from './bracket_highlight';

	export let code_document_rect: LogicalFrame;
	export let annotation_section: LogicalFrame;
	export let active_window_uid: number;

	let top_rectangle: LogicalFrame | null = null;
	let bottom_rectangle: LogicalFrame | null = null;

	let annotation_group_id: string | null = null;
	let annotation_group_editor_window_uid: number | null = null;

	let opening_bracket_box: LogicalFrame | null = null;
	let closing_bracket_box: LogicalFrame | null = null;

	const listen_to_node_annotation_events = async () => {
		let node_annotation_channel: ChannelList = 'NodeAnnotationEvent';
		await listen(node_annotation_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(event.payload as string) as AnnotationEvent;
			switch (event_type) {
				case 'AddAnnotationGroup':
				case 'UpdateAnnotationGroup':
					if (payload.feature === 'BracketHighlight') {
						let group = payload;
						annotation_group_editor_window_uid = group.editor_window_uid;

						let closing_bracket = try_get_kind_as_rectangle(group, 'ClosingBracket');
						if (closing_bracket) {
							closing_bracket_box = closing_bracket;
						}

						let opening_bracket = try_get_kind_as_rectangle(group, 'OpeningBracket');
						if (opening_bracket) {
							opening_bracket_box = opening_bracket;
						}
						const rectangles = get_elbow_rectangles_from_annotation_group(group);
						top_rectangle = rectangles.top_rectangle;
						bottom_rectangle = rectangles.bottom_rectangle;
					}
					break;
				case 'RemoveAnnotationGroup':
					let group_id = payload as string;

					if (annotation_group_id === group_id) {
						annotation_group_editor_window_uid = null;
						annotation_group_id = null;

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

	interface BracketHighlightRectangles {
		top_rectangle: LogicalFrame | null;
		bottom_rectangle: LogicalFrame | null;
	}

	function get_elbow_pos(elbow_shape: AnnotationShape | undefined): LogicalPosition {
		const default_elbow_pos = {
			x: annotation_section.origin.x - code_document_rect.origin.x + annotation_section.size.width,
			y: 0
		};
		return is_point(elbow_shape) ? elbow_shape.Point : default_elbow_pos;
	}

	const get_elbow_rectangles_from_annotation_group = (
		group: AnnotationGroup
	): BracketHighlightRectangles => {
		const lines_start = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'LineStart'
		);

		const lines_end = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'LineEnd'
		);

		const elbow = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'Elbow'
		);

		const lines_start_pos =
			lines_start && is_point(lines_start.shapes[0]) ? lines_start.shapes[0].Point : null;

		const lines_end_pos =
			lines_end && is_point(lines_end.shapes[0]) ? lines_end.shapes[0].Point : null;

		const elbow_pos = elbow ? get_elbow_pos(elbow.shapes[0]) : null;

		const rectangles = compute_bracket_highlight_lines(
			lines_start_pos,
			lines_end_pos,
			elbow_pos,
			code_document_rect.size.height
		);

		top_rectangle = rectangles.top_rect;
		bottom_rectangle = rectangles.bottom_rect;

		return {
			top_rectangle,
			bottom_rectangle
		};
	};

	const round_value = (value: number, precision: number): number => {
		const factor = Math.pow(10, precision || 0);
		return Math.round(value * factor) / factor;
	};
</script>

{#if annotation_group_editor_window_uid === active_window_uid}
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
	{#if closing_bracket_box && code_document_rect}
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
	{#if top_rectangle}
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
	{#if bottom_rectangle}
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
