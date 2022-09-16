<script>
	import Button from "../common/button.svelte";
	import IconThumbsDown from "./icons/icon-thumbs-down.svelte";
	import IconThumbsUp from "./icons/icon-thumbs-up.svelte";

</script>
<div class="flex justify-between w-full">
    <div class="flex gap-3 px-1">
        <IconThumbsUp/>
        <IconThumbsDown/>
    </div>
    <Button>Insert as Docstring</Button>
</div>