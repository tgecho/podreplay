import initWasm, * as wasm from 'podreplay_lib_wasm';
import { fromUnixTime, getUnixTime } from 'date-fns';
import { once } from 'lodash-es';

const init = once(initWasm);

export async function reschedule(
  items: { timestamp: string }[],
  rule: string,
  start: string,
  first?: string,
  last?: string,
): Promise<(null | Date)[]> {
  await init();

  const timestamps = new Float64Array(items.length);
  for (let i = 0; i < items.length; i++) {
    timestamps[i] = getUnixTime(new Date(items[i].timestamp));
  }

  const rescheduled = await wasm.reschedule(
    timestamps,
    rule,
    getUnixTime(new Date(start)),
    first ? getUnixTime(new Date(first)) : undefined,
    last ? getUnixTime(new Date(last)) : undefined,
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
