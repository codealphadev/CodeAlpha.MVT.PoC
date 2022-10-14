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
		.sort((a, b) => a[0].localeCompare(b[0]));
};
