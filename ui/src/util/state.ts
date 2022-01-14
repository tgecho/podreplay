import { debounce } from 'lodash-es';
import { page } from '$app/stores';
import { goto } from '$app/navigation';
import { derived, get, writable } from 'svelte/store';
import type { Writable, Readable } from 'svelte/store';
import { parseRule, ruleToString } from './parseRule';
import type { Rule } from './parseRule';
import { formatForUrl } from './dates';

export function sortedQueryString(s: State): string {
  const rule = ruleToString(s);
  return [
    s.start && `start=${formatForUrl(s.start)}`,
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
    start: new Date(Date.parse(query.get('start') || '') || Date.now()),
    first: query.get('first'),
    last: query.get('last'),
    title: query.get('title'),
    ...parseRule(query.get('rule')),
  };
}

export type State = Rule & {
  uri: string;
  start: Date;
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
      goto(`${location.pathname}?${queryString}`, {
        replaceState: true,
        keepfocus: true,
        noscroll: true,
      });
    }, 100),
  );

  return store;
}

export function replayUrlStore(queryStore: Readable<State>) {
  return derived(queryStore, (query) => {
    const queryString = sortedQueryString(query);
    return `${location.origin}/replay?${queryString}`;
  });
}
