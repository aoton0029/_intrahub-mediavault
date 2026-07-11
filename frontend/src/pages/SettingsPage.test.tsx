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
  const importBooklog = vi.fn();
  const importSteam = vi.fn();
  const fetchHealth = vi.fn();

  beforeEach(() => {
    saveApiKey.mockReset();
    importBooklog.mockReset();
    importSteam.mockReset();
    fetchHealth.mockReset();
    fetchHealth.mockResolvedValue({ database: "ok" });

    mockUseSettingsData.mockReturnValue({
      saveApiKey,
      importBooklog,
      importSteam,
      fetchHealth,
    });
  });

  afterEach(() => {
    mockUseSettingsData.mockReset();
  });

  it("renders three tabs and starts on the API tab", () => {
    render(<SettingsPage />);

    expect(screen.getByRole("button", { name: "API連携" })).toHaveClass("active");
    expect(screen.getByRole("button", { name: "データインポート" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "システム状態" })).toBeInTheDocument();
    expect(screen.getByText("外部データソースのAPIキーを登録します。各プロバイダごとに個別のキーを保存できます。")).toBeInTheDocument();
  });

  it("shows seven provider rows and renders Jikan as read-only", () => {
    render(<SettingsPage />);

    expect(screen.getAllByText(/provider:/)).toHaveLength(7);
    expect(screen.queryAllByPlaceholderText("APIキーを入力")).toHaveLength(6);
    expect(screen.getByText("Jikan(MyAnimeList)")).toBeInTheDocument();
    expect(screen.getByText("設定不要")).toBeInTheDocument();
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
  });

  it("imports a Booklog CSV and shows the latest import result", async () => {
    const user = userEvent.setup();
    importBooklog.mockResolvedValue({
      successCount: 10,
      failureCount: 2,
      failures: [
        { row: 5, reason: "タイトルが空です" },
        { row: 9, reason: "不正な評価値です" },
      ],
    });
    render(<SettingsPage />);

    await user.click(screen.getByRole("button", { name: "データインポート" }));
    const fileInput = screen.getByLabelText("CSVファイル") as HTMLInputElement;
    const file = new File(["title"], "booklog.csv", { type: "text/csv" });
    await user.upload(fileInput, file);
    await user.click(screen.getByRole("button", { name: "アップロードして取り込む" }));

    await waitFor(() => {
      expect(importBooklog).toHaveBeenCalledWith(file);
    });
    expect(screen.getByText("直近のインポート結果")).toBeInTheDocument();
    expect(screen.getByText("成功: 10件")).toBeInTheDocument();
    expect(screen.getByText("失敗: 2件")).toBeInTheDocument();
    expect(screen.getByText("5行目")).toBeInTheDocument();
    expect(screen.getByText("reason: タイトルが空です")).toBeInTheDocument();
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
