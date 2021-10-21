import { isAfter } from 'date-fns/esm';
import type { FeedItem } from './fetchFeedSummary';

export function shiftItems(items: FeedItem[], start: Date) {
  const before = [];
  const after = [];
  for (const item of items) {
    if (isAfter(start, item.timestamp)) {
      after.push(item);
    } else {
      before.push(item);
    }
  }
  const shifted = [];
}
