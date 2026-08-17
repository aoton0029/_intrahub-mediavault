import { useState } from "react";
import { toast } from "sonner";
import { Modal } from "./Modal";
import { useThemeSongSearch, createThemeSong } from "@/hooks/useThemeSongSearch";
import { themeSongTypeLabels } from "@/config/themeSongLabels";

export type ThemeSongTypeValue = "op" | "ed" | "insert" | "image" | "character" | "theme" | "other";

/** バックエンドの ThemeSongType enum と同じ並び順（表示順もこれに従う） */
const THEME_TYPE_ORDER: ThemeSongTypeValue[] = ["op", "ed", "insert", "image", "character", "theme", "other"];

export function ThemeSongAddModal({
  open,
  onClose,
  onLink,
}: {
  open: boolean;
  onClose: () => void;
  onLink: (themeSongId: string, themeType: ThemeSongTypeValue, displayOrder?: number) => Promise<unknown>;
}) {
  const [title, setTitle] = useState("");
  const { results, isLoading } = useThemeSongSearch(title);

  const [selectedSongId, setSelectedSongId] = useState<string | null>(null);
  const [selectedSongTitle, setSelectedSongTitle] = useState("");

  const [showNewSongForm, setShowNewSongForm] = useState(false);
  const [newSongTitle, setNewSongTitle] = useState("");
  const [newSongArtist, setNewSongArtist] = useState("");
  const [newSongComposer, setNewSongComposer] = useState("");
  const [newSongLyricist, setNewSongLyricist] = useState("");
  const [newSongArranger, setNewSongArranger] = useState("");
  const [newSongNote, setNewSongNote] = useState("");

  const [themeType, setThemeType] = useState<ThemeSongTypeValue>("op");
  const [displayOrder, setDisplayOrder] = useState("");
  const [isSubmitting, setSubmitting] = useState(false);

  function selectSong(songId: string, songTitle: string) {
    setSelectedSongId(songId);
    setSelectedSongTitle(songTitle);
    setShowNewSongForm(false);
  }

  async function handleSubmit() {
    setSubmitting(true);
    try {
      let themeSongId = selectedSongId;
      if (showNewSongForm || !themeSongId) {
        if (!newSongTitle.trim()) {
          toast.error("曲名を入力するか、既存の曲を選択してください。");
          setSubmitting(false);
          return;
        }
        const song = await createThemeSong({
          title: newSongTitle,
          artist: newSongArtist,
          composer: newSongComposer,
          lyricist: newSongLyricist,
          arranger: newSongArranger,
          note: newSongNote,
        });
        themeSongId = song.id;
      }

      const order = displayOrder.trim() === "" ? undefined : Number(displayOrder);
      if (order !== undefined && Number.isNaN(order)) {
        toast.error("表示順は数値で入力してください。");
        setSubmitting(false);
        return;
      }

      await onLink(themeSongId!, themeType, order);
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "テーマソングの追加に失敗しました。");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal open={open} onClose={onClose} title="テーマソングを追加" maxWidth={560}>
      <div className="search-box" style={{ marginBottom: 14 }}>
        🔍 <input
          type="text"
          placeholder="曲名で検索…"
          value={title}
          onChange={(event) => setTitle(event.target.value)}
        />
      </div>

      {isLoading ? <p>検索中です...</p> : null}
      {title.trim().length > 0 && !isLoading && results.length === 0 ? <p>該当する曲が見つかりませんでした。</p> : null}
      {results.map((song) => (
        <div key={song.id} className="result-row">
          <div className="info">
            <div className="title">{song.title}</div>
            <div className="sub">{song.artist || "アーティスト未登録"}</div>
          </div>
          <button
            type="button"
            className={selectedSongId === song.id ? "btn btn-sm" : "btn btn-accent btn-sm"}
            disabled={selectedSongId === song.id}
            onClick={() => selectSong(song.id, song.title)}
          >
            {selectedSongId === song.id ? "選択済み" : "選択"}
          </button>
        </div>
      ))}

      <hr className="rail-divider" />

      <button type="button" className="tag-add-trigger" style={{ marginBottom: 12 }} onClick={() => setShowNewSongForm((value) => !value)}>
        + 新規テーマソングを登録
      </button>
      {showNewSongForm ? (
        <div className="form-grid" style={{ marginBottom: 18 }}>
          <div className="form-field full">
            <label>
              曲名 <span className="required">*</span>
            </label>
            <input type="text" placeholder="例: 残酷な天使のテーゼ" value={newSongTitle} onChange={(event) => setNewSongTitle(event.target.value)} />
          </div>
          <div className="form-field">
            <label>アーティスト</label>
            <input type="text" placeholder="任意" value={newSongArtist} onChange={(event) => setNewSongArtist(event.target.value)} />
          </div>
          <div className="form-field">
            <label>作曲</label>
            <input type="text" placeholder="任意" value={newSongComposer} onChange={(event) => setNewSongComposer(event.target.value)} />
          </div>
          <div className="form-field">
            <label>作詞</label>
            <input type="text" placeholder="任意" value={newSongLyricist} onChange={(event) => setNewSongLyricist(event.target.value)} />
          </div>
          <div className="form-field">
            <label>編曲</label>
            <input type="text" placeholder="任意" value={newSongArranger} onChange={(event) => setNewSongArranger(event.target.value)} />
          </div>
          <div className="form-field full">
            <label>備考</label>
            <input type="text" placeholder="任意" value={newSongNote} onChange={(event) => setNewSongNote(event.target.value)} />
          </div>
        </div>
      ) : null}

      <hr className="rail-divider" />

      <div className="form-grid">
        <div className="form-field">
          <label>
            種別 <span className="required">*</span>
          </label>
          <select value={themeType} onChange={(event) => setThemeType(event.target.value as ThemeSongTypeValue)}>
            {THEME_TYPE_ORDER.map((value) => (
              <option key={value} value={value}>
                {themeSongTypeLabels[value]}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>表示順</label>
          <input type="number" placeholder="任意" value={displayOrder} onChange={(event) => setDisplayOrder(event.target.value)} />
        </div>
      </div>

      {selectedSongId && !showNewSongForm ? <p style={{ marginTop: 10, fontSize: 12 }}>選択中: {selectedSongTitle}</p> : null}

      <div className="form-actions">
        <button type="button" className="btn btn-accent" disabled={isSubmitting} onClick={() => void handleSubmit()}>
          追加
        </button>
        <button type="button" className="btn btn-ghost" onClick={onClose}>
          キャンセル
        </button>
      </div>
    </Modal>
  );
}
