<script lang="ts">
	import type { RefactoringOperation } from "../../../src-tauri/bindings/features/refactoring/RefactoringOperation";
	import Diff from "../common/diff.svelte";

	import { emit } from "@tauri-apps/api/event";
	import type { ChannelList } from '../../../src-tauri/bindings/ChannelList';
	import type { EventUserInteraction } from '../../../src-tauri/bindings/user_interaction/EventUserInteraction';
	import Button from "../common/button.svelte";

    export let suggestion: RefactoringOperation;    

    const apply_suggestion = async () => {
        const event: EventUserInteraction = {event: 'PerformRefactoringOperation', payload: {id: suggestion.id}};
        const channel: ChannelList = 'EventUserInteractions';
        
        await emit(channel, event)
    }

</script>

<div class="flex flex-col items-start p-3 gap-2 w-full rounded-xl border-backgroundsecondary">
    <Diff old_code={suggestion.old_text_content_string} new_code={suggestion.new_text_content_string}  />
    <div class="flex justify-end w-full items-center">
       
        <Button on:click={apply_suggestion}>Apply</Button>
    </div>
</div>
	
