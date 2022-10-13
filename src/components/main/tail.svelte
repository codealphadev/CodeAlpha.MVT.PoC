<script lang="ts">
  import { Event, listen } from '@tauri-apps/api/event';
  import { colors } from '../../themes';
  import type { TailOrientation } from '../../../src-tauri/bindings/window_controls/TailOrientation';

  let tail_orientation: TailOrientation = 'Right';
  const listenToGlobalEvents = async () => {
    await listen('tail-orientation-changed', (event) => {
      tail_orientation = (event as Event<any>).payload as TailOrientation;
    });
  };

  listenToGlobalEvents();
</script>

<div class="h-3 {`${tail_orientation == 'Left' ? 'mr-auto ml-[18px]' : 'ml-auto mr-[18px]'}`}">
  <svg width="36" height="12" viewBox="0 0 36 12" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path
      d="M18 12C19.5215 12 21.5225 9.98632 23.8851 7.60873C27.2899 4.18226 31.4458 0 36 0H0C4.55419 0 8.71006 4.18226 12.1149 7.60873C14.4775 9.98632 16.4785 12 18 12Z"
      fill={colors.background}
    />
  </svg>
</div>
