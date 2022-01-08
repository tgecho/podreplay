import { derived, get, readable } from 'svelte/store';
import type { Readable } from 'svelte/store';
import type { State } from './state';

export type FeedItem = {
  id: string;
  title: string;
  timestamp: string;
};

export type FeedSummary = {
  title: string;
  uri: string;
  items: FeedItem[];
};

export function fetchFeedSummary(uri?: string, fetchFn = fetch): Promise<FeedSummary> {
  if (uri?.trim()) {
    return fetchFn(`/summary?uri=${uri}`).then((r) => {
      if (r.status === 200) {
        return r.json();
      } else {
        return r.text().then((text) => Promise.reject(text));
      }
    });
  } else {
    return Promise.reject(new Error(`Missing a Feed URL`));
  }
}

export function feedSummaryStore(state: Readable<State>): Readable<Promise<FeedSummary>> {
  return derived(
    derivedWithCaching(state, (s) => s.uri),
    (uri) => fetchFeedSummary(uri),
  );
}

/* A wrapper around derived that only emits values when they change. Uses === for change detection. */
function derivedWithCaching<A, B>(store: Readable<A>, fn: (a: A) => B): Readable<B> {
  let last = fn(get(store));
  return readable(last, (send) => {
    return store.subscribe((s) => {
      const next = fn(s);
      if (next !== last) {
        last = next;
        send(next);
      }
    });
  });
}

function dbg<V>(v: V): V {
  console.trace(v);
  return v;
}
