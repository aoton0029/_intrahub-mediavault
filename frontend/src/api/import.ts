import { useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from './client';
import type { ImportSummary } from '@/types';

export async function importBooklog(file: File): Promise<ImportSummary> {
  const formData = new FormData();
  formData.append('file', file);
  const { data } = await apiClient<ImportSummary>('/import/booklog', {
    method: 'POST',
    body: formData,
  });
  return data;
}

export async function importSteam(steamId: string): Promise<ImportSummary> {
  const { data } = await apiClient<ImportSummary>('/import/steam', {
    method: 'POST',
    body: { steamId },
  });
  return data;
}

export function useImportBooklogMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: importBooklog,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items'] });
    },
  });
}

export function useImportSteamMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: importSteam,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items'] });
    },
  });
}
