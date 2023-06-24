import { debounce } from 'lodash-es';
import { page } from '$app/stores';
import { goto } from '$app/navigation';
import { browser } from '$app/environment';

import { get, writable } from 'svelte/store';
import type { Writable } from 'svelte/store';
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
  const query = browser ? get(page).url.searchParams : new URLSearchParams();

  const store = writable<State>(queryToState(query));

  if (browser) {
    store.subscribe(
      debounce((query) => {
        const queryString = sortedQueryString(query);
        goto(`${location.pathname}?${queryString}`, {
          replaceState: true,
          keepFocus: true,
          noScroll: true,
        });
      }, 100),
    );
  }

  return store;
}
