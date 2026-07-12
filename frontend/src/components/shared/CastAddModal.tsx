import { useState } from "react";
import { toast } from "sonner";
import { Modal } from "./Modal";
import { useCastSearch, createCast } from "@/hooks/useCastSearch";

export function CastAddModal({
  open,
  onClose,
  onLink,
}: {
  open: boolean;
  onClose: () => void;
  onLink: (castId: string, characterName?: string) => Promise<unknown>;
}) {
  const [name, setName] = useState("");
  const { results, isLoading } = useCastSearch(name);

  const [selectedCastId, setSelectedCastId] = useState<string | null>(null);
  const [selectedCastName, setSelectedCastName] = useState("");

  const [showNewCastForm, setShowNewCastForm] = useState(false);
  const [newCastName, setNewCastName] = useState("");
  const [newCastExternalId, setNewCastExternalId] = useState("");
  const [newCastImageUrl, setNewCastImageUrl] = useState("");

  const [characterName, setCharacterName] = useState("");
  const [isSubmitting, setSubmitting] = useState(false);

  function selectCast(castId: string, castName: string) {
    setSelectedCastId(castId);
    setSelectedCastName(castName);
    setShowNewCastForm(false);
  }

  async function handleSubmit() {
    setSubmitting(true);
    try {
      let castId = selectedCastId;
      if (showNewCastForm || !castId) {
        if (!newCastName.trim()) {
          toast.error("氏名を入力するか、既存キャストを選択してください。");
          setSubmitting(false);
          return;
        }
        const cast = await createCast({ name: newCastName, externalId: newCastExternalId, imageUrl: newCastImageUrl });
        castId = cast.id;
      }

      await onLink(castId!, characterName || undefined);
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "キャストの追加に失敗しました。");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal open={open} onClose={onClose} title="キャストを追加" maxWidth={560}>
      <div className="search-box" style={{ marginBottom: 14 }}>
        🔍 <input
          type="text"
          placeholder="氏名で検索…"
          value={name}
          onChange={(event) => setName(event.target.value)}
        />
      </div>

      {isLoading ? <p>検索中です...</p> : null}
      {name.trim().length > 0 && !isLoading && results.length === 0 ? <p>該当するキャストが見つかりませんでした。</p> : null}
      {results.map((cast) => (
        <div key={cast.id} className="result-row">
          <div className="thumb" style={{ borderRadius: 999, width: 44, height: 44 }} />
          <div className="info">
            <div className="title">{cast.name}</div>
            <div className="sub">紐付け作品 {cast.linked_item_count}件</div>
          </div>
          <button
            type="button"
            className={selectedCastId === cast.id ? "btn btn-sm" : "btn btn-accent btn-sm"}
            disabled={selectedCastId === cast.id}
            onClick={() => selectCast(cast.id, cast.name)}
          >
            {selectedCastId === cast.id ? "選択済み" : "選択"}
          </button>
        </div>
      ))}

      <hr className="rail-divider" />

      <button type="button" className="tag-add-trigger" style={{ marginBottom: 12 }} onClick={() => setShowNewCastForm((value) => !value)}>
        + 新規キャストを登録
      </button>
      {showNewCastForm ? (
        <div className="form-grid" style={{ marginBottom: 18 }}>
          <div className="form-field full">
            <label>
              氏名 <span className="required">*</span>
            </label>
            <input type="text" placeholder="例: 結城 かなで" value={newCastName} onChange={(event) => setNewCastName(event.target.value)} />
          </div>
          <div className="form-field">
            <label>external_id</label>
            <input type="text" placeholder="任意" value={newCastExternalId} onChange={(event) => setNewCastExternalId(event.target.value)} />
          </div>
          <div className="form-field">
            <label>image_url</label>
            <input type="text" placeholder="任意" value={newCastImageUrl} onChange={(event) => setNewCastImageUrl(event.target.value)} />
          </div>
        </div>
      ) : null}

      <hr className="rail-divider" />

      <div className="form-grid">
        <div className="form-field full">
          <label>キャラ名</label>
          <input type="text" placeholder="例: ルカ(任意)" value={characterName} onChange={(event) => setCharacterName(event.target.value)} />
        </div>
      </div>

      {selectedCastId && !showNewCastForm ? <p style={{ marginTop: 10, fontSize: 12 }}>選択中: {selectedCastName}</p> : null}

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
