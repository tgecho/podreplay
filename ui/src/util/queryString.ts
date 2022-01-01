import { sortBy } from 'lodash-es';

const keys = ['rule', 'first', 'last', 'start', 'uri'];
const sortFn = keys.indexOf.bind(keys);

export function sortedQueryString(query: URLSearchParams): string {
  return sortBy(Array.from(query.keys()), sortFn)
    .map((k) => `${k}=${query.get(k)}`)
    .join('&');
}
