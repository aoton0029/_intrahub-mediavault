/**
 * PUT /settings/api-keys/{provider} の対象プロバイダ（docs/backend/mediavault-api/settings.md）。
 * jikanはAPIキー不要のため対象外。
 */
export type ApiProvider = 'tmdb' | 'igdb' | 'ndl' | 'steam' | 'open_library' | 'ani_list';

export const apiProviders: ApiProvider[] = ['tmdb', 'igdb', 'ndl', 'steam', 'open_library', 'ani_list'];

export const apiProviderLabels: Record<ApiProvider, string> = {
  tmdb: 'TMDB',
  igdb: 'IGDB',
  ndl: 'NDL(国立国会図書館)',
  steam: 'Steam',
  open_library: 'Open Library',
  ani_list: 'AniList',
};

export interface ApiCredential {
  provider: ApiProvider;
  api_key: string;
  updated_at: string;
}

export interface ImportFailure {
  row_number: number;
  reason: string;
}

export interface ImportSummary {
  success_count: number;
  failure_count: number;
  failures: ImportFailure[];
}

export interface HealthStatus {
  status: string;
}
