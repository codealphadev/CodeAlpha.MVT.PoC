<script lang="ts">
	import { invoke } from "@tauri-apps/api/tauri";
	import Button from "../common/button.svelte";
	import IconButton from "../common/icon-button.svelte";
	import IconThumbsDown from "./icons/icon-thumbs-down.svelte";
	import IconThumbsUp from "./icons/icon-thumbs-up.svelte";
    
    const paste_docs = async () => {
        console.log('Hello');
		await invoke('cmd_paste_docs');
	}

    const vote_good = async () => {
        await invoke('send-feedback', {
            feature: 'NodeExplainer',
            feedback: 'good'
        })
    }

    const vote_bad = async () => {
        await invoke('send-feedback', {
            feature: 'NodeExplainer',
            feedback: 'bad'
        })
    }
</script>

<div class="flex justify-between w-full items-center">
    <div class="flex gap-3 px-1 ">
        <IconButton on:click={vote_good}>
            <IconThumbsUp/>
        </IconButton>
        <IconButton on:click={vote_bad}>
            <IconThumbsDown/>
        </IconButton>
    </div>
    <Button on:click={paste_docs}>Insert as Docstring</Button>
</div>