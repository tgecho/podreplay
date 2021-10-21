import { page } from '$app/stores';
import { goto } from '$app/navigation';

export function queryStore(name: string) {
  let query;
  return {
    subscribe: (fn) => {
      return page.subscribe((page) => {
        query = page.query;
        fn(page.query.get(name));
      });
    },
    set: (value) => {
      console.log('query.set', query, name, value);
      query?.set(name, value);
      goto(`?${query.toString()}`, { keepfocus: true, replaceState: true, noscroll: true });
    },
  };
}
