import initWasm, * as wasm from 'podreplay_lib_wasm';
import { fromUnixTime, getUnixTime } from 'date-fns';
import { once } from 'lodash-es';
import type { State } from './state';
import type { FeedSummary } from './fetchFeedSummary';
import { ruleToString } from './parseRule';

export const init = once(initWasm);

export type Rescheduled = (Date | null)[];

export function reschedule(feed: FeedSummary, state: State): Rescheduled {
  const { items } = feed;

  const timestamps = new Float64Array(items.length);
  for (let i = 0; i < items.length; i++) {
    timestamps[i] = getUnixTime(new Date(items[i].timestamp));
  }

  const rule = ruleToString(state);

  const rescheduled = wasm.reschedule(
    timestamps,
    rule,
    getUnixTime(state.start),
    state.first ? getUnixTime(new Date(state.first)) : undefined,
    state.last ? getUnixTime(new Date(state.last)) : undefined,
  );

  const results = [];
  for (const timestamp of rescheduled) {
    if (timestamp !== 0) {
      results.push(fromUnixTime(timestamp));
    } else {
      results.push(null);
    }
  }
  return results;
}
