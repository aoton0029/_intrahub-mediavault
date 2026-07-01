import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from './client';
import type { ApiCredential, ApiProvider, UpsertApiCredentialRequest } from '@/types';

export async function fetchApiCredentials(): Promise<ApiCredential[]> {
  const { data } = await apiClient<ApiCredential[]>('/settings/api-keys');
  return data;
}

export async function upsertApiCredential(
  provider: ApiProvider,
  body: UpsertApiCredentialRequest
): Promise<ApiCredential> {
  const { data } = await apiClient<ApiCredential>(`/settings/api-keys/${provider}`, {
    method: 'PUT',
    body,
  });
  return data;
}

const QUERY_KEY = ['settings', 'api-credentials'] as const;

export function useApiCredentialsQuery() {
  return useQuery({
    queryKey: QUERY_KEY,
    queryFn: fetchApiCredentials,
  });
}

export function useUpsertApiCredentialMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ provider, apiKey }: { provider: ApiProvider; apiKey: string }) =>
      upsertApiCredential(provider, { apiKey }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEY });
    },
  });
}
