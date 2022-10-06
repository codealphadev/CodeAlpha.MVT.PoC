<script lang="ts">
	//import Diff from "../common/diff.svelte";
	import H3 from '../common/typography/h3.svelte';
	import { emit } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../src-tauri/bindings/ChannelList';
	import type { EventUserInteraction } from '../../../src-tauri/bindings/user_interaction/EventUserInteraction';
	import Button from '../common/button.svelte';
	import Card from '../common/card.svelte';
	import P from '../common/typography/p.svelte';
	import H4 from '../common/typography/h4.svelte';
	import Arrow from './icons/arrow.svelte';
	import type { FERefactoringSuggestion } from '../../../src-tauri/bindings/features/refactoring/FERefactoringSuggestion';
	import IconButton from '../common/icon-button.svelte';
	import IconRubbishBin from './icons/icon-rubbish-bin.svelte';

	export let suggestion: FERefactoringSuggestion;

	const apply_suggestion = async () => {
		const event: EventUserInteraction = {
			event: 'PerformRefactoringOperation',
			payload: { id: suggestion.id, editor_window_uid: suggestion.window_uid }
		};
		const channel: ChannelList = 'EventUserInteractions';

		await emit(channel, event);
		//<div class="flex flex-col items-start p-3 gap-2 w-full rounded-xl border-backgroundsecondary">
		//    <Diff old_code={suggestion.old_text_content_string} new_code={suggestion.new_text_content_string}  />
	};

	const dismiss_suggestion = async () => {
		const event: EventUserInteraction = {
			event: 'DismissRefactoringSuggestion',
			payload: { id: suggestion.id, editor_window_uid: suggestion.window_uid }
		};
		const channel: ChannelList = 'EventUserInteractions';
		console.log('DISMISSING SUGGESTION');
		await emit(channel, event);
	};

	function map_complexity_to_text(complexity: number): { text: string; class: string } {
		if (complexity < 10) {
			return { text: `low (${complexity})`, class: 'text-signalgood' };
		} else if (complexity < 15) {
			return { text: `medium (${complexity})`, class: 'text-signalmedium' };
		} else if (complexity < 20) {
			return { text: `high (${complexity})`, class: 'text-signalbad' };
		} else {
			return { text: `very high (${complexity})`, class: 'text-signalverybad' };
		}
	}
	$: prev_complexity = map_complexity_to_text(suggestion.prev_complexity);
	$: new_complexity = map_complexity_to_text(suggestion.new_complexity);

	function title_case(str: string) {
		if (str.length === 0) {
			return str;
		}
		return str[0]!.toUpperCase() + str.slice(1).toLowerCase();
	}
</script>

<Card>
	<header>
		<H3>Reduce complexity</H3>
		<P>
			Your function <code>{suggestion.main_function_name}</code> may be hard to understand due to nested
			statements. Consider extracting this code block into a separate function.
		</P>
	</header>
	<H4>Impact on complexity</H4>
	<div class="flex items-center gap-1">
		<div class="text-sm font-bold {prev_complexity.class}">
			{title_case(prev_complexity.text)}
		</div>
		<Arrow />
		<div class="text-sm font-bold {new_complexity.class}">
			{new_complexity.text}
		</div>
	</div>

	<div class="flex justify-between w-full items-center">
		<IconButton on:click={dismiss_suggestion}>
			<div
				class="p-2 border-secondary rounded bg-backgroundsecondary"
				style=".hover.hover: brighten(25%)"
			>
				<IconRubbishBin />
			</div>
		</IconButton>
		<Button on:click={apply_suggestion}>Extract function</Button>
	</div>
</Card>
