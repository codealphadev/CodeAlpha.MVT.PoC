import { writable } from "svelte/store";
import { Event, listen } from '@tauri-apps/api/event';
import type { ChannelList } from '../src-tauri/bindings/ChannelList';
import type { RuleMatch } from '../src-tauri/bindings/rules/RuleMatch';
import type { RuleResults } from '../src-tauri/bindings/rules/RuleResults';

export const alerts = writable([]);


const listenToGlobalEvents = async () => {
  let ruleMatches: RuleMatch[] = [];
  await listen('RuleResults' as ChannelList, (event) => {
    const tauriEvent = event as Event<any>;
    let ruleResults: RuleResults[] = tauriEvent.payload;
    ruleMatches = [];
    for (let ruleResult of ruleResults) {
      ruleMatches = ruleMatches.concat(ruleResult.results);
    }
    alerts.set(ruleMatches);
  });
};

listenToGlobalEvents();
