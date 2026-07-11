import { apiFetch } from "@/lib/apiClient";

export type ApiCredential = {
  provider: string;
  apiKey: string;
  updatedAt: string;
};

export type ImportSummary = {
  successCount: number;
  failureCount: number;
  failures: { row: number; reason: string }[];
};

export type HealthStatus = {
  database: "ok" | "error";
};

type ApiEnvelope<T> = {
  success?: boolean;
  data?: T;
  error?: {
    code?: string;
    message?: string;
  };
};

function getErrorMessage(payload: unknown, fallback: string) {
  if (typeof payload === "object" && payload !== null && "error" in payload) {
    const error = (payload as ApiEnvelope<unknown>).error;
    if (error?.message) {
      return error.code ? `${error.code}: ${error.message}` : error.message;
    }
  }

  return fallback;
}

async function readJson(response: Response) {
  try {
    return await response.json();
  } catch {
    return null;
  }
}

type ImportSummaryPayload = {
  success_count?: number;
  failure_count?: number;
  failures?: { row_number: number; reason: string }[];
};

function mapImportSummary(data: ImportSummaryPayload): ImportSummary {
  return {
    successCount: data.success_count ?? 0,
    failureCount: data.failure_count ?? 0,
    failures: (data.failures ?? []).map((failure) => ({
      row: failure.row_number,
      reason: failure.reason,
    })),
  };
}

function extractImportSummaryPayload(payload: ApiEnvelope<ImportSummaryPayload> | ImportSummaryPayload | null): ImportSummaryPayload | null {
  if (!payload) {
    return null;
  }

  if ("success" in payload || "error" in payload || "data" in payload) {
    return payload.data ?? null;
  }

  return payload as ImportSummaryPayload;
}

export async function saveApiKey(provider: string, apiKey: string): Promise<ApiCredential> {
  const response = await apiFetch(`/settings/api-keys/${provider}`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ api_key: apiKey }),
  });
  const payload = (await readJson(response)) as ApiEnvelope<{
    provider: string;
    api_key: string;
    updated_at: string;
  }> | null;

  if (!response.ok || !payload?.data) {
    throw new Error(getErrorMessage(payload, "APIキーの保存に失敗しました"));
  }

  return {
    provider: payload.data.provider,
    apiKey: payload.data.api_key,
    updatedAt: payload.data.updated_at,
  };
}

export async function importBooklog(file: File): Promise<ImportSummary> {
  const formData = new FormData();
  formData.append("file", file);

  const response = await apiFetch("/import/booklog", {
    method: "POST",
    body: formData,
  });
  const payload = (await readJson(response)) as ApiEnvelope<ImportSummaryPayload> | ImportSummaryPayload | null;

  if (!response.ok) {
    throw new Error(getErrorMessage(payload, "Booklogの取り込みに失敗しました"));
  }

  const data = extractImportSummaryPayload(payload);
  if (!data) {
    throw new Error("Booklogの取り込み結果を取得できませんでした");
  }

  return mapImportSummary(data);
}

export async function importSteam(steamId: string): Promise<ImportSummary> {
  const response = await apiFetch("/import/steam", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ steam_id: steamId }),
  });
  const payload = (await readJson(response)) as ApiEnvelope<ImportSummaryPayload> | null;

  if (!response.ok || !payload?.data) {
    throw new Error(getErrorMessage(payload, "Steamライブラリの取り込みに失敗しました"));
  }

  return mapImportSummary(payload.data);
}

export async function fetchHealth(): Promise<HealthStatus> {
  const response = await apiFetch("/health");
  const payload = (await readJson(response)) as ApiEnvelope<{ status: "ok" | "error" }> | null;

  if (!response.ok || !payload?.data) {
    throw new Error(getErrorMessage(payload, "システム状態の取得に失敗しました"));
  }

  return {
    database: payload.data.status === "ok" ? "ok" : "error",
  };
}

export function useSettingsData() {
  return {
    saveApiKey,
    importBooklog,
    importSteam,
    fetchHealth,
  };
}
