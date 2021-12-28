import init, * as wasm from 'podreplay_lib_wasm';
import { fromUnixTime, getUnixTime } from 'date-fns';

export async function reschedule(items: { timestamp: string }[]): Promise<(null | Date)[]> {
  await init();

  const timestamps = new Float64Array(items.length);
  for (let i = 0; i < items.length; i++) {
    timestamps[i] = getUnixTime(new Date(items[i].timestamp));
  }

  const rescheduled = await wasm.reschedule(
    timestamps,
    getUnixTime(new Date('2021-05-03T14:00:00Z')),
  );

  const results = [];
  for (const timestamp of rescheduled) {
    if (timestamp !== 0) {
      results.push(fromUnixTime(timestamp));
    } else {
      results.push(null);
    }
  }
  return results;
}
