const RE = /^(\d+)(m|w|d)(Su)?(M)?(Tu)?(W)?(Th)?(F)?(Sa)?/;

export type Rule = {
  interval: number;
  freq: 'm' | 'w' | 'd';
  weekdays: {
    Su?: boolean;
    M?: boolean;
    Tu?: boolean;
    W?: boolean;
    Th?: boolean;
    F?: boolean;
    Sa?: boolean;
  };
};
type Weekday = keyof Rule['weekdays'];

export function parseRule(rule: string): Rule {
  const ruleMatch = RE.exec(rule);
  if (ruleMatch) {
    const [, interval, freq, Su, M, Tu, W, Th, F, Sa] = ruleMatch;
    return {
      interval: parseInt(interval) || 1,
      freq: freq as Rule['freq'],
      weekdays: {
        Su: Boolean(Su),
        M: Boolean(M),
        Tu: Boolean(Tu),
        W: Boolean(W),
        Th: Boolean(Th),
        F: Boolean(F),
        Sa: Boolean(Sa),
      },
    };
  }
  return { interval: 1, freq: 'w', weekdays: {} };
}

const days: Weekday[] = ['Su', 'M', 'Tu', 'W', 'Th', 'F', 'Sa'];

export function ruleToString(rule: Rule): string {
  return `${rule.interval}${rule.freq}${days.filter((d) => rule.weekdays[d]).join('')}`;
}
