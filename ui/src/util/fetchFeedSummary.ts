export type FeedItem = { title: string; timestamp: number };
export type FeedSummary = {
  title: string;
  items: FeedItem[];
};
export function fetchFeedSummary(uri: string, fetchFn = fetch): Promise<FeedSummary> {
  return fetchFn(`http://localhost:3030/api/summary?uri=${uri}`).then((r) => r.json());
}
