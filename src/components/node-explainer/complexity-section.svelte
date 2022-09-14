<script lang="ts">

    export let complexity: number;
</script>
<div>
    <div class="absolute rounded-md bg-indigo-500 p-3">
      <!-- Heroicon name: outline/users -->
      <svg class="h-6 w-6 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z" />
      </svg>
    </div>
    <p class="ml-16 truncate text-sm font-medium text-gray-500">Complexity</p>
</div>
  <div class="ml-16 flex items-baseline pb-6 sm:pb-7">
    <p class="text-2xl font-semibold text-gray-900">{complexity}</p>
    <p class="ml-2 flex items-baseline text-sm font-semibold text-green-600">
      <!-- Heroicon name: mini/arrow-up -->
      <svg class="h-5 w-5 flex-shrink-0 self-center text-green-500" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
        <path fill-rule="evenodd" d="M10 17a.75.75 0 01-.75-.75V5.612L5.29 9.77a.75.75 0 01-1.08-1.04l5.25-5.5a.75.75 0 011.08 0l5.25 5.5a.75.75 0 11-1.08 1.04l-3.96-4.158V16.25A.75.75 0 0110 17z" clip-rule="evenodd" />
      </svg>
      <span class="sr-only"> Too high by </span>
      122
    </p>
</div>