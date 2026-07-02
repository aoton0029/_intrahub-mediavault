import { ApiClientError } from '@/types';
import type { ApiResponse, Pagination } from '@/types';

const BASE_URL = import.meta.env.VITE_API_BASE_URL ?? '/api/v1';

interface RequestOptions {
  method?: 'GET' | 'POST' | 'PATCH' | 'DELETE' | 'PUT';
  body?: unknown;
  signal?: AbortSignal;
  headers?: Record<string, string>;
}

/**
 * バックエンドの `/api/v1` 配下のみを対象とするfetchラッパー。
 * `/internal/*` の呼び出しは設計上の制約（REQ-402）により禁止されているため、呼び出し側で使用しないこと。
 */
export async function apiClient<T>(
  path: string,
  options: RequestOptions = {}
): Promise<{ data: T; pagination?: Pagination }> {
  const { method = 'GET', body, signal, headers } = options;

  let res: Response;
  try {
    res = await fetch(`${BASE_URL}${path}`, {
      method,
      headers: {
        ...(body !== undefined && !(body instanceof FormData)
          ? { 'Content-Type': 'application/json' }
          : {}),
        ...headers,
      },
      body: body instanceof FormData ? body : body !== undefined ? JSON.stringify(body) : undefined,
      signal,
    });
  } catch (err) {
    throw new ApiClientError(
      'NETWORK_ERROR',
      err instanceof Error ? err.message : 'ネットワークエラーが発生しました'
    );
  }

  let json: ApiResponse<T>;
  try {
    json = await res.json();
  } catch {
    throw new ApiClientError('PARSE_ERROR', 'レスポンスの解析に失敗しました');
  }

  if (!json.success) {
    throw new ApiClientError(json.error.code, json.error.message);
  }

  return { data: json.data, pagination: json.pagination };
}
