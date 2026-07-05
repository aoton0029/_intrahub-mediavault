import { useMutation, useQuery } from '@tanstack/react-query';
import { apiFetch } from '@/lib/api-client';
import type { ApiCredential, ApiProvider, HealthStatus, ImportSummary } from './types';

export async function updateApiKey(provider: ApiProvider, apiKey: string): Promise<ApiCredential> {
  return (
    await apiFetch<ApiCredential>(`/settings/api-keys/${provider}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ api_key: apiKey }),
    })
  ).data;
}

export function useUpdateApiKeyMutation() {
  return useMutation({
    mutationFn: ({ provider, apiKey }: { provider: ApiProvider; apiKey: string }) =>
      updateApiKey(provider, apiKey),
  });
}

export async function importBooklog(file: File): Promise<ImportSummary> {
  const formData = new FormData();
  formData.set('file', file);
  return (
    await apiFetch<ImportSummary>('/import/booklog', {
      method: 'POST',
      body: formData,
    })
  ).data;
}

export function useImportBooklogMutation() {
  return useMutation({ mutationFn: importBooklog });
}

export async function importSteam(steamId: string): Promise<ImportSummary> {
  return (
    await apiFetch<ImportSummary>('/import/steam', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ steam_id: steamId }),
    })
  ).data;
}

export function useImportSteamMutation() {
  return useMutation({ mutationFn: importSteam });
}

export function useHealthQuery() {
  return useQuery({
    queryKey: ['health'],
    queryFn: async () => (await apiFetch<HealthStatus>('/health')).data,
    retry: false,
  });
}
