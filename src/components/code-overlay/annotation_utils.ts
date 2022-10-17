import type { AnnotationGroup } from '../../../src-tauri/bindings/features/code_annotations/AnnotationGroup';
import type { AnnotationKind } from '../../../src-tauri/bindings/features/code_annotations/AnnotationKind';
import type { AnnotationShape } from '../../../src-tauri/bindings/features/code_annotations/AnnotationShape';
import type { LogicalFrame } from '../../../src-tauri/bindings/geometry/LogicalFrame';
import type { LogicalPosition } from '../../../src-tauri/bindings/geometry/LogicalPosition';

export function is_rectangle(
	shape: AnnotationShape | undefined
): shape is { Rectangle: LogicalFrame } {
	return shape?.hasOwnProperty('Rectangle') ?? false;
}

export function is_point(shape: AnnotationShape | undefined): shape is { Point: LogicalPosition } {
	return shape?.hasOwnProperty('Point') ?? false;
}

export function try_get_kind_as_rectangle(
	group: AnnotationGroup,
	kind: AnnotationKind
): LogicalFrame | undefined {
	let result = Object.values(group.annotations).find((annotation) => annotation.kind === kind);
	if (!result || !result.shapes[0] || !is_rectangle(result.shapes[0])) {
		return;
	}
	return result.shapes[0].Rectangle;
}

export const round_value = (value: number, precision: number): number => {
	const factor = Math.pow(10, precision || 0);
	return Math.round(value * factor) / factor;
};
