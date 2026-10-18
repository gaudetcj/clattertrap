// The query string a page is opened with, and the one it writes back.

export function param(search: string, name: string): string | undefined {
  const value = new URLSearchParams(search).get(name);
  return value === null || value === '' ? undefined : value;
}

export function search(params: Record<string, string>): string {
  const entries = Object.entries(params);
  if (entries.length === 0) {
    return '';
  }
  return `?${entries.map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`).join('&')}`;
}
