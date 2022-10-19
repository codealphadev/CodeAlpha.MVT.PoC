import type { FERefactoringSuggestion } from '../../../src-tauri/bindings/features/refactoring/FERefactoringSuggestion';
import type { ReplaceSuggestionsMessage } from '../../../src-tauri/bindings/features/refactoring/ReplaceSuggestionsMessage';

export const filter_and_sort_suggestions = (
	suggestions: ReplaceSuggestionsMessage['suggestions'],
	active_window_uid: number | null
): [string, FERefactoringSuggestion][] => {
	if (!active_window_uid) {
		return [];
	}
	return Object.entries(suggestions[active_window_uid] ?? {})
		.filter(([_, value]) => value.state !== 'New')
		.sort((a, b) => a[1].start_index - b[1].start_index);
};

export const every_suggestion_is_new = (
	suggestions: ReplaceSuggestionsMessage['suggestions'],
	active_window_uid: number | null
): boolean => {
	if (!active_window_uid) {
		return false;
	}
	let suggestions_for_window = Object.entries(suggestions[active_window_uid] ?? {}).filter(
		([_, value]) => value.state == 'New'
	);
	return (
		suggestions_for_window.length > 0 &&
		suggestions_for_window.every(([_, suggestion]) => suggestion.state == 'New')
	);
};
