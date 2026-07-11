const API_BASE = "/api/v1";

export function apiFetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
  if (typeof input === "string") {
    return fetch(`${API_BASE}${input}`, init);
  }
  return fetch(input, init);
}
