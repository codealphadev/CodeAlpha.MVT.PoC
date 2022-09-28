<script lang="ts">
	import { invoke } from "@tauri-apps/api/tauri";
	import Button from "../common/button.svelte";
	import IconButton from "../common/icon-button.svelte";
	import IconThumbsDown from "./icons/icon-thumbs-down.svelte";
	import IconThumbsUp from "./icons/icon-thumbs-up.svelte";
    import { appWindow } from '@tauri-apps/api/window';
	import { emit } from "@tauri-apps/api/event";


    let vote: 'good' | 'bad' | null = null;
    const paste_docs = async () => {
		await invoke('cmd_paste_docs');
        await appWindow.hide();

	}

    const refactor = async () => {
        await emit('EventUserInteractions', {
            event: 'PerformRefactoringOperation',
            payload: {
            }
        }) // TODO type it properly
        console.log('emittedit')
    }

    const handle_click_thumbs_up = async () => {
        if (vote) {
            return;
        }
        await invoke('cmd_send_feedback', {
            feature: 'NodeExplainer',
            feedback: 'good'
        });
        vote = 'good';
    }

    const handle_click_thumbs_down = async () => {
        if (vote) {
            return;
        }
        await invoke('cmd_send_feedback', {
            feature: 'NodeExplainer',
            feedback: 'bad'
        })
        await appWindow.hide();
        vote = 'bad';
    }
</script>

<div class="flex justify-between w-full items-center">
    <div class="flex gap-3 px-1 items-center">
        <IconButton on:click={handle_click_thumbs_up}>
            <IconThumbsUp activated={vote === 'good'}/>
        </IconButton>
        <IconButton on:click={handle_click_thumbs_down}>
            <IconThumbsDown activated={vote === 'bad'}/>
        </IconButton>   
        <div class="text-secondary text-xs">
            {#if vote}
            Thanks for the feedback!
            {:else}
            Honestly, was this helpful?
            {/if}
        </div>
        
    </div>
    <Button on:click={refactor}>Refactor</Button>
    <Button on:click={paste_docs}>Insert as Docstring</Button>
</div>