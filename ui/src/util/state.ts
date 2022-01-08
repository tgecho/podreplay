import { debounce } from 'lodash-es';
import { page } from '$app/stores';
import { goto } from '$app/navigation';
import { derived, get, readable, writable } from 'svelte/store';
import type { Writable, Readable } from 'svelte/store';
import { parseRule, ruleToString } from './parseRule';
import type { Rule } from './parseRule';
import { reschedule } from './reschedule';
import type { FeedSummary } from './fetchFeedSummary';

export function sortedQueryString(s: State): string {
  const rule = ruleToString(s);
  return [
    rule && `rule=${rule}`,
    s.first && `first=${s.first}`,
    s.last && `last=${s.last}`,
    s.title && `title=${encodeURIComponent(s.title)}`,
    s.uri && `uri=${encodeURIComponent(s.uri)}`,
  ]
    .filter(Boolean)
    .join('&');
}

export function queryToState(query: URLSearchParams): State {
  return {
    uri: query.get('uri') || '',
    first: query.get('first'),
    last: query.get('last'),
    title: query.get('title'),
    ...parseRule(query.get('rule')),
  };
}

export type State = Rule & {
  uri: string;
  first: string | null;
  last: string | null;
  title: string | null;
};

export function queryStore(): Writable<State> {
  const query = get(page).url.searchParams;

  const store = writable<State>(queryToState(query));

  store.subscribe(
    debounce((query) => {
      // TODO: tighten up the formatted date to not include sub second precision
      const queryString = sortedQueryString(query);
      goto(`${location.pathname}?${queryString}`, { replaceState: true, keepfocus: true });
    }, 100),
  );

  return store;
}

export function replayUrlStore(queryStore: Readable<State>, start: string) {
  return derived(queryStore, (query) => {
    const queryString = sortedQueryString(query);
    return `${location.origin}/replay?start=${start}&${queryString}`;
  });
}

export type Rescheduled = (Date | null)[];

export function rescheduledStore(feed: FeedSummary, start: string, queryStore: Readable<State>) {
  return readable<Rescheduled>([], (set) => {
    queryStore.subscribe((query) => {
      const rule = ruleToString(query);
      reschedule(feed.items, rule, start, query.first || undefined, query.last || undefined).then(
        set,
      );
    });
  });
}
