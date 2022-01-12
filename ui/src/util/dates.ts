import { format } from 'date-fns';

export function formatForInput(date: Date) {
  return format(date, "yyyy-MM-dd'T'HH:mm");
}

export function formatForUrl(date: Date) {
  return format(date, "yyyy-MM-dd'T'HH:mm:ss'Z'");
}
