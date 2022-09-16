
<script lang="ts">
	import { gradient_tailwind } from "../../themes";
</script>

<button on:click type="button" class=" cursor-pointer
        inline-flex items-center justify-center py-2 px-3 h-8 rounded
        border-none shadow-md 
        {gradient_tailwind}
        hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-[#054b8b] 
        text-white text-xs font-medium">
    <slot>

    </slot>
</button>