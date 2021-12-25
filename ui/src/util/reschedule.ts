import type { FeedSummary } from './fetchFeedSummary';
import init, * as wasm from 'podreplay_lib_wasm';
import { fromUnixTime, getUnixTime } from 'date-fns';

export async function reschedule(feed: FeedSummary): Promise<(null | Date)[]> {
  await init();

  const items = new Float64Array(feed.items.length);
  for (let i = 0; i < feed.items.length; i++) {
    items[i] = getUnixTime(new Date(feed.items[i].timestamp));
  }

  const timestamps = await wasm.reschedule(
    items,
    getUnixTime(new Date('2021-05-03T14:00:00Z')),
    getUnixTime(new Date()),
    getUnixTime(new Date('2020-10-03T14:00:00Z')),
  );

  const results = [];
  for (const timestamp of timestamps) {
    if (timestamp !== 0) {
      results.push(fromUnixTime(timestamp));
    } else {
      results.push(null);
    }
  }
  return results;
}
