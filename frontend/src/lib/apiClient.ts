import type { ApiErrorCode, Pagination } from '@/features/items/types';

/**
 * MediaVault API 共通fetchラッパー
 *
 * ベースURLは `/api/v1`。認証ヘッダーは付与しない（REQ-401/NFR-101）。
 * ApiOk<T> / PaginatedOk<T> のエンベロープから実データを抽出し、
 * success: false のエラーレスポンスは ApiClientError としてthrowする。
 *
 * 関連: docs/tasks/collection-browsing/TASK-0004.md
 */

const API_BASE_URL = '/api/v1';

/**
 * APIエラーレスポンス（success: false）由来のエラー。
 * 呼び出し側は `code` を見て EDGE-001〜003 等の分岐処理を行う。
 */
export class ApiClientError extends Error {
  readonly code: ApiErrorCode;

  constructor(code: ApiErrorCode, message: string) {
    super(message);
    this.name = 'ApiClientError';
    this.code = code;
  }
}

function buildUrl(path: string): string {
  const normalizedPath = path.startsWith('/') ? path : `/${path}`;
  return `${API_BASE_URL}${normalizedPath}`;
}

interface ApiOkBody<T, P> {
  success: true;
  data: T;
  pagination?: P;
}

interface ApiErrorBody {
  success: false;
  error: {
    code: ApiErrorCode;
    message: string;
  };
}

/**
 * ページネーション付きレスポンスの抽出結果。
 */
export interface PaginatedResult<T, P = Pagination> {
  data: T;
  pagination: P;
}

/**
 * MediaVault APIへリクエストを送り、レスポンスエンベロープから実データを抽出する。
 *
 * - `ApiOk<T>` の場合: `data` をそのまま返す。
 * - `PaginatedOk<T>` の場合（レスポンスに `pagination` を含む場合）:
 *   `{ data, pagination }` を返す。
 * - `success: false` の場合: `ApiClientError` をthrowする。
 * - fetch自体の失敗・JSON parse失敗時は `ApiClientError`（code: 'INTERNAL_ERROR'）としてthrowする。
 *
 * @param path - `/api/v1` からの相対パス（先頭のスラッシュは省略可）
 * @param options - fetchにそのまま渡すオプション
 */
export async function apiRequest<T, P = Pagination>(
  path: string,
  options?: RequestInit,
): Promise<T> {
  let response: Response;
  try {
    response = await fetch(buildUrl(path), options);
  } catch (cause) {
    throw new ApiClientError(
      'INTERNAL_ERROR',
      cause instanceof Error ? cause.message : 'Network request failed',
    );
  }

  let body: ApiOkBody<T, P> | ApiErrorBody;
  try {
    body = (await response.json()) as ApiOkBody<T, P> | ApiErrorBody;
  } catch {
    throw new ApiClientError('INTERNAL_ERROR', 'Malformed JSON response');
  }

  if (!body.success) {
    throw new ApiClientError(body.error.code, body.error.message);
  }

  if (body.pagination !== undefined) {
    return { data: body.data, pagination: body.pagination } as unknown as T;
  }

  return body.data;
}
