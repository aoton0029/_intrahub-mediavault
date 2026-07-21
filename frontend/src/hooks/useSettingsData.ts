import { apiFetch } from "@/lib/apiClient";

export type ApiCredential = {
  provider: string;
  apiKey: string;
  updatedAt: string;
};

export type ApiKeyStatus = {
  provider: string;
  configured: boolean;
};

export type ImportSummary = {
  successCount: number;
  failureCount: number;
  failures: { row: number; reason: string }[];
};

export type ImportJobStatus = {
  status: "running" | "cancelling" | "cancelled" | "completed";
  total: number;
  processed: number;
  summary: ImportSummary | null;
};

export type HealthStatus = {
  database: "ok" | "error";
};

export type BackupTableCount = {
  inserted: number;
  skipped: number;
};

export type BackupImportReport = {
  tables: Record<string, BackupTableCount>;
  totalInserted: number;
  totalSkipped: number;
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

type ImportJobStatusPayload = {
  status?: "running" | "cancelling" | "cancelled" | "completed";
  total?: number;
  processed?: number;
  summary?: ImportSummaryPayload | null;
};

function mapImportJobStatus(status: ImportJobStatusPayload["status"]): ImportJobStatus["status"] {
  if (status === "completed" || status === "cancelling" || status === "cancelled") {
    return status;
  }
  return "running";
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

export async function fetchApiKeyStatuses(): Promise<ApiKeyStatus[]> {
  const response = await apiFetch("/settings/api-keys");
  const payload = (await readJson(response)) as ApiEnvelope<ApiKeyStatus[]> | null;

  if (!response.ok || !payload?.data) {
    throw new Error(getErrorMessage(payload, "APIキーの設定状況を取得できませんでした"));
  }

  return payload.data;
}

/** Booklog CSVをアップロードしてインポートジョブを起動し、job_idを返す */
export async function startBooklogImport(file: File): Promise<string> {
  const formData = new FormData();
  formData.append("file", file);

  const response = await apiFetch("/import/booklog", {
    method: "POST",
    body: formData,
  });
  const payload = (await readJson(response)) as ApiEnvelope<{ job_id?: string }> | null;

  if (!response.ok || !payload?.data?.job_id) {
    throw new Error(getErrorMessage(payload, "Booklogの取り込み開始に失敗しました"));
  }

  return payload.data.job_id;
}

/** ブクログインポートジョブの進捗・完了後の結果を取得する（ポーリング用） */
export async function fetchBooklogImportJob(jobId: string): Promise<ImportJobStatus> {
  const response = await apiFetch(`/import/booklog/jobs/${jobId}`);
  const payload = (await readJson(response)) as ApiEnvelope<ImportJobStatusPayload> | null;

  if (!response.ok || !payload?.data) {
    throw new Error(getErrorMessage(payload, "インポート進捗の取得に失敗しました"));
  }

  return {
    status: mapImportJobStatus(payload.data.status),
    total: payload.data.total ?? 0,
    processed: payload.data.processed ?? 0,
    summary: payload.data.summary ? mapImportSummary(payload.data.summary) : null,
  };
}

/** 実行中のブクログインポートジョブへ中止をリクエストする */
export async function cancelBooklogImportJob(jobId: string): Promise<void> {
  const response = await apiFetch(`/import/booklog/jobs/${jobId}/cancel`, {
    method: "POST",
  });

  if (!response.ok) {
    const payload = await readJson(response);
    throw new Error(getErrorMessage(payload, "インポートの中止に失敗しました"));
  }
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

/** GET /backup/export のレスポンスをそのままJSONファイルとしてダウンロードする */
export async function exportBackup(): Promise<void> {
  const response = await apiFetch("/backup/export");

  if (!response.ok) {
    const payload = await readJson(response);
    throw new Error(getErrorMessage(payload, "バックアップのエクスポートに失敗しました"));
  }

  const disposition = response.headers.get("Content-Disposition");
  const filename = disposition?.match(/filename="([^"]+)"/)?.[1] ?? "mediavault-backup.json";

  const blob = await response.blob();
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

type BackupImportReportPayload = {
  tables?: Record<string, { inserted?: number; skipped?: number }>;
  total_inserted?: number;
  total_skipped?: number;
};

/** バックアップJSONファイルをPOST /backup/importへそのまま送信する */
export async function importBackup(file: File): Promise<BackupImportReport> {
  const response = await apiFetch("/backup/import", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: file,
  });
  const payload = (await readJson(response)) as ApiEnvelope<BackupImportReportPayload> | null;

  if (!response.ok || !payload?.data) {
    throw new Error(getErrorMessage(payload, "バックアップのインポートに失敗しました"));
  }

  return {
    tables: Object.fromEntries(
      Object.entries(payload.data.tables ?? {}).map(([table, count]) => [
        table,
        { inserted: count.inserted ?? 0, skipped: count.skipped ?? 0 },
      ]),
    ),
    totalInserted: payload.data.total_inserted ?? 0,
    totalSkipped: payload.data.total_skipped ?? 0,
  };
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
    fetchApiKeyStatuses,
    startBooklogImport,
    fetchBooklogImportJob,
    cancelBooklogImportJob,
    importSteam,
    exportBackup,
    importBackup,
    fetchHealth,
  };
}
