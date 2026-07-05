const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:8080/api/v1';

export class ApiClientError extends Error {
  code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = 'ApiClientError';
    this.code = code;
  }
}

interface ApiOkEnvelope<T> {
  success: true;
  data: T;
}

interface PaginatedOkEnvelope<T> extends ApiOkEnvelope<T> {
  pagination: { limit: number; has_more: boolean; next_after_created_at: string | null; next_after_id: string | null };
}

interface ApiErrorEnvelope {
  success: false;
  error: { code: string; message: string };
}

async function parseEnvelope<T>(path: string, response: Response): Promise<ApiOkEnvelope<T> | PaginatedOkEnvelope<T>> {
  let body: ApiOkEnvelope<T> | PaginatedOkEnvelope<T> | ApiErrorEnvelope;
  try {
    body = await response.json();
  } catch (e) {
    if (import.meta.env.DEV) console.error(`[API] ${path} → JSON parse failed`, e);
    throw new ApiClientError('INTERNAL_ERROR', '一覧の取得に失敗しました。再試行してください。');
  }

  if (import.meta.env.DEV) {
    if (!response.ok || body.success === false) {
      console.error(`[API] ${path} → ${response.status}`, body);
    } else {
      console.log(`[API] ${path} → ${response.status}`, body);
    }
  }

  if (!response.ok || body.success === false) {
    const errorBody = (body as ApiErrorEnvelope).error;
    throw new ApiClientError(
      errorBody?.code ?? 'INTERNAL_ERROR',
      errorBody?.message ?? '一覧の取得に失敗しました。再試行してください。',
    );
  }

  return body;
}

export async function apiFetch<T>(path: string, init?: RequestInit): Promise<{ data: T }> {
  const response = await fetch(`${API_BASE_URL}${path}`, init);
  const body = await parseEnvelope<T>(path, response);
  return { data: body.data };
}

export async function apiFetchPaginated<T>(
  path: string,
  init?: RequestInit,
): Promise<{ data: T; pagination: { limit: number; has_more: boolean; next_after_created_at: string | null; next_after_id: string | null } }> {
  const response = await fetch(`${API_BASE_URL}${path}`, init);
  const body = await parseEnvelope<T>(path, response);
  if (!('pagination' in body)) {
    throw new ApiClientError('INTERNAL_ERROR', 'ページネーション情報が含まれていません');
  }
  return { data: body.data, pagination: body.pagination };
}
