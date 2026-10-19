// Where a query string written for an older landing page belongs: a machine link to the level
// page, a lineup link to the arena, anything else stays home.

import { param } from './query';

export function forwardFor(query: string): string | undefined {
  if (param(query, 'entry') !== undefined) {
    return `level.html${query}`;
  }
  if (param(query, 'level') !== undefined || param(query, 'arm') !== undefined) {
    return `arena.html${query}`;
  }
  return undefined;
}
