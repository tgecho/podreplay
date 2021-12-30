export type FeedItem = { title: string; timestamp: string };
export type FeedSummary = {
  title: string;
  uri: string;
  items: FeedItem[];
};
export function fetchFeedSummary(uri: string, fetchFn = fetch): Promise<FeedSummary> {
  return fetchFn(`http://localhost:3000/summary?uri=${uri}`).then((r) => r.json());
}
