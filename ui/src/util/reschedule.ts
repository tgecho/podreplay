import type { FeedSummary } from './fetchFeedSummary';

export function reschedule(feed: FeedSummary) {
  return import('podreplay_lib_wasm').then(async (m) => {
    await m.default();
    return m.reschedule(
      feed,
      '2020-10-03T14:00:00Z',
      '2021-10-03T14:00:00Z',
      '2020-10-03T14:00:00Z',
    );
  });
}
