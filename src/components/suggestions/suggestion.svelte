<script lang="ts">
	import H3 from '../common/typography/h3.svelte';
	import { emit } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../src-tauri/bindings/ChannelList';
	import type { EventUserInteraction } from '../../../src-tauri/bindings/user_interaction/EventUserInteraction';
	import Button from '../common/buttons/button.svelte';
	import Card from '../common/card.svelte';
	import H4 from '../common/typography/h4.svelte';
	import Arrow from './icons/arrow.svelte';
	import type { FERefactoringSuggestion } from '../../../src-tauri/bindings/features/refactoring/FERefactoringSuggestion';
	import IconButton from '../common/buttons/icon-button.svelte';
	import IconRubbishBin from './icons/icon-rubbish-bin.svelte';
	import ComplexityBadge from './complexity-badge.svelte';

	export let suggestion: FERefactoringSuggestion;
	export let suggestion_id: string;
	export let window_uid: number;
	export let expanded = false;

	const apply_suggestion = async () => {
		const event: EventUserInteraction = {
			event: 'PerformRefactoringOperation',
			payload: { id: suggestion_id, editor_window_uid: window_uid }
		};
		const channel: ChannelList = 'EventUserInteractions';

		await emit(channel, event);
	};

	const dismiss_suggestion = async () => {
		const event: EventUserInteraction = {
			event: 'DismissSuggestion',
			payload: { id: suggestion_id, editor_window_uid: window_uid }
		};
		const channel: ChannelList = 'EventUserInteractions';
		await emit(channel, event);
	};
</script>

<Card on:click>
	<header>
		<H3>Reduce complexity</H3>
		<p class="text-contrast text-sm leading-[1.714]">
			{#if expanded}
				Your function <code>{suggestion.main_function_name}</code> may be hard to understand due to nested
				statements. Consider extracting this code block into a separate function.
			{:else}
				Refactor deeply nested statements in function <code>{suggestion.main_function_name}</code>
			{/if}
		</p>
	</header>
	{#if expanded}
		<div class="flex flex-col shrink-0 gap-3 items-start">
			<H4>Impact on complexity</H4>
			<div class="flex items-center gap-1">
				<ComplexityBadge complexity={suggestion.prev_complexity} />
				<Arrow />
				<ComplexityBadge complexity={suggestion.new_complexity} />
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
		</div>
	{/if}
</Card>
