import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SettingsPage } from "./SettingsPage";
import { useSettingsData } from "@/hooks/useSettingsData";

vi.mock("@/hooks/useSettingsData", () => ({
  useSettingsData: vi.fn(),
}));

const mockUseSettingsData = vi.mocked(useSettingsData);

describe("SettingsPage", () => {
  const saveApiKey = vi.fn();
  const fetchApiKeyStatuses = vi.fn();
  const startBooklogImport = vi.fn();
  const fetchBooklogImportJob = vi.fn();
  const cancelBooklogImportJob = vi.fn();
  const importSteam = vi.fn();
  const exportBackup = vi.fn();
  const importBackup = vi.fn();
  const fetchHealth = vi.fn();

  beforeEach(() => {
    saveApiKey.mockReset();
    fetchApiKeyStatuses.mockReset();
    startBooklogImport.mockReset();
    fetchBooklogImportJob.mockReset();
    cancelBooklogImportJob.mockReset();
    importSteam.mockReset();
    exportBackup.mockReset();
    importBackup.mockReset();
    fetchHealth.mockReset();
    fetchHealth.mockResolvedValue({ database: "ok" });
    fetchApiKeyStatuses.mockResolvedValue([]);

    mockUseSettingsData.mockReturnValue({
      saveApiKey,
      fetchApiKeyStatuses,
      startBooklogImport,
      fetchBooklogImportJob,
      cancelBooklogImportJob,
      importSteam,
      exportBackup,
      importBackup,
      fetchHealth,
    });
  });

  afterEach(() => {
    mockUseSettingsData.mockReset();
  });

  it("renders four tabs and starts on the API tab", () => {
    render(<SettingsPage />);

    expect(screen.getByRole("button", { name: "API連携" })).toHaveClass("active");
    expect(screen.getByRole("button", { name: "データインポート" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "バックアップ" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "システム状態" })).toBeInTheDocument();
    expect(screen.getByText("外部データソースのAPIキーを登録します。各プロバイダごとに個別のキーを保存できます。")).toBeInTheDocument();
  });

  it("shows only the four providers that use API keys", () => {
    render(<SettingsPage />);

    expect(screen.getAllByText(/provider:/)).toHaveLength(4);
    expect(screen.getAllByPlaceholderText("APIキーを入力")).toHaveLength(4);
    expect(screen.queryByText("IGDB")).not.toBeInTheDocument();
    expect(screen.queryByText("NDL(国立国会図書館)")).not.toBeInTheDocument();
    expect(screen.queryByText("Open Library")).not.toBeInTheDocument();
    expect(screen.queryByText("Jikan(MyAnimeList)")).not.toBeInTheDocument();
  });

  it("loads configured API-key providers from the backend", async () => {
    fetchApiKeyStatuses.mockResolvedValue([
      { provider: "tmdb", configured: true },
      { provider: "steam", configured: false },
    ]);

    render(<SettingsPage />);

    expect(await screen.findByText("provider: tmdb ・ 設定済み")).toBeInTheDocument();
    expect(screen.getByText("provider: steam ・ 未設定")).toBeInTheDocument();
  });

  it("saves an API key with the matching provider and value", async () => {
    const user = userEvent.setup();
    saveApiKey.mockResolvedValue({ provider: "tmdb", apiKey: "test-key", updatedAt: "2026-07-11T12:00:00" });
    render(<SettingsPage />);

    await user.type(screen.getByLabelText("TMDB APIキー"), "test-key");
    await user.click(screen.getAllByRole("button", { name: "保存" })[0]);

    await waitFor(() => {
      expect(saveApiKey).toHaveBeenCalledWith("tmdb", "test-key");
    });
    expect(screen.getByText("provider: tmdb ・ 設定済み")).toBeInTheDocument();
  });

  it("imports a Booklog CSV and shows the latest import result after the job completes", async () => {
    const user = userEvent.setup();
    startBooklogImport.mockResolvedValue("job-123");
    fetchBooklogImportJob.mockResolvedValueOnce({
      status: "completed",
      total: 12,
      processed: 12,
      summary: {
        successCount: 10,
        failureCount: 2,
        failures: [
          { row: 5, reason: "タイトルが空です" },
          { row: 9, reason: "不正な評価値です" },
        ],
      },
    });
    render(<SettingsPage />);

    await user.click(screen.getByRole("button", { name: "データインポート" }));
    const fileInput = screen.getByLabelText("CSVファイル") as HTMLInputElement;
    const file = new File(["title"], "booklog.csv", { type: "text/csv" });
    await user.upload(fileInput, file);
    await user.click(screen.getByRole("button", { name: "アップロードして取り込む" }));

    await waitFor(() => {
      expect(startBooklogImport).toHaveBeenCalledWith(file);
    });
    await waitFor(() => {
      expect(fetchBooklogImportJob).toHaveBeenCalledWith("job-123");
    });
    expect(await screen.findByText("直近のインポート結果")).toBeInTheDocument();
    expect(screen.getByText("成功: 10件")).toBeInTheDocument();
    expect(screen.getByText("失敗: 2件")).toBeInTheDocument();
    expect(screen.getByText("5行目")).toBeInTheDocument();
    expect(screen.getByText("reason: タイトルが空です")).toBeInTheDocument();
  });

  it("shows progress while the Booklog import job is still running", async () => {
    const user = userEvent.setup();
    startBooklogImport.mockResolvedValue("job-456");
    fetchBooklogImportJob
      .mockResolvedValueOnce({ status: "running", total: 100, processed: 30, summary: null })
      .mockResolvedValueOnce({
        status: "completed",
        total: 100,
        processed: 100,
        summary: { successCount: 100, failureCount: 0, failures: [] },
      });
    render(<SettingsPage />);

    await user.click(screen.getByRole("button", { name: "データインポート" }));
    const fileInput = screen.getByLabelText("CSVファイル") as HTMLInputElement;
    const file = new File(["title"], "booklog.csv", { type: "text/csv" });
    await user.upload(fileInput, file);
    await user.click(screen.getByRole("button", { name: "アップロードして取り込む" }));

    expect(await screen.findByText("30 / 100件処理済み（楽天ブックスAPIの制限により時間がかかる場合があります）")).toBeInTheDocument();
    expect(await screen.findByText("成功: 100件")).toBeInTheDocument();
  });

  it("shows a cancel button while the Booklog import job is running and cancels it", async () => {
    const user = userEvent.setup();
    startBooklogImport.mockResolvedValue("job-789");
    cancelBooklogImportJob.mockResolvedValue(undefined);
    fetchBooklogImportJob
      .mockResolvedValueOnce({ status: "running", total: 100, processed: 10, summary: null })
      .mockResolvedValueOnce({ status: "cancelled", total: 100, processed: 12, summary: { successCount: 10, failureCount: 2, failures: [] } });
    render(<SettingsPage />);

    await user.click(screen.getByRole("button", { name: "データインポート" }));
    const fileInput = screen.getByLabelText("CSVファイル") as HTMLInputElement;
    const file = new File(["title"], "booklog.csv", { type: "text/csv" });
    await user.upload(fileInput, file);
    await user.click(screen.getByRole("button", { name: "アップロードして取り込む" }));

    const cancelButton = await screen.findByRole("button", { name: "中止" });
    await user.click(cancelButton);

    await waitFor(() => {
      expect(cancelBooklogImportJob).toHaveBeenCalledWith("job-789");
    });
    expect(await screen.findByText("取り込みを中止しました")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "中止" })).not.toBeInTheDocument();
  });

  it("imports a Steam library with the entered Steam ID", async () => {
    const user = userEvent.setup();
    importSteam.mockResolvedValue({ successCount: 2, failureCount: 0, failures: [] });
    render(<SettingsPage />);

    await user.click(screen.getByRole("button", { name: "データインポート" }));
    await user.type(screen.getByRole("textbox", { name: "Steam ID" }), "76561198000000000");
    await user.click(screen.getByRole("button", { name: "ライブラリを取り込む" }));

    await waitFor(() => {
      expect(importSteam).toHaveBeenCalledWith("76561198000000000");
    });
  });

  it("exports a backup file from the backup tab", async () => {
    const user = userEvent.setup();
    exportBackup.mockResolvedValue(undefined);
    render(<SettingsPage />);

    await user.click(screen.getByRole("button", { name: "バックアップ" }));
    await user.click(screen.getByRole("button", { name: "エクスポート" }));

    await waitFor(() => {
      expect(exportBackup).toHaveBeenCalled();
    });
  });

  it("imports a backup file and shows the per-table report", async () => {
    const user = userEvent.setup();
    importBackup.mockResolvedValue({
      tables: {
        items: { inserted: 3, skipped: 1 },
        tags: { inserted: 0, skipped: 0 },
      },
      totalInserted: 3,
      totalSkipped: 1,
    });
    render(<SettingsPage />);

    await user.click(screen.getByRole("button", { name: "バックアップ" }));
    const fileInput = screen.getByLabelText("バックアップファイル") as HTMLInputElement;
    const file = new File(['{"schema_version":1}'], "mediavault-backup.json", { type: "application/json" });
    await user.upload(fileInput, file);
    await user.click(screen.getByRole("button", { name: "インポート" }));

    await waitFor(() => {
      expect(importBackup).toHaveBeenCalledWith(file);
    });
    expect(screen.getByText("直近のインポート結果")).toBeInTheDocument();
    expect(screen.getByText("追加: 3件")).toBeInTheDocument();
    expect(screen.getByText("スキップ（既存）: 1件")).toBeInTheDocument();
    expect(screen.getByText("items")).toBeInTheDocument();
    // 件数0のテーブルは一覧に出さない
    expect(screen.queryByText("tags")).not.toBeInTheDocument();
  });

  it("shows an error when importing a backup without selecting a file", async () => {
    const user = userEvent.setup();
    render(<SettingsPage />);

    await user.click(screen.getByRole("button", { name: "バックアップ" }));
    await user.click(screen.getByRole("button", { name: "インポート" }));

    expect(importBackup).not.toHaveBeenCalled();
    expect(screen.getByText("バックアップファイル（JSON）を選択してください")).toBeInTheDocument();
  });

  it("fetches health status and shows the ok pill styling", async () => {
    render(<SettingsPage />);
    screen.getByRole("button", { name: "システム状態" }).click();

    await waitFor(() => {
      expect(fetchHealth).toHaveBeenCalled();
    });

    const pill = await screen.findByText("status: ok");
    expect(pill).toHaveClass("tag-pill");
    expect(pill).toHaveStyle({ color: "var(--color-status-done)" });
  });
});
