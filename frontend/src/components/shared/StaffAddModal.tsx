import { useState } from "react";
import { toast } from "sonner";
import { Modal } from "./Modal";
import { useStaffSearch, createStaff } from "@/hooks/useStaffSearch";

export function StaffAddModal({
  open,
  onClose,
  onLink,
}: {
  open: boolean;
  onClose: () => void;
  onLink: (staffId: string, role: string, characterName?: string) => Promise<unknown>;
}) {
  const [name, setName] = useState("");
  const { results, isLoading } = useStaffSearch(name);

  const [selectedStaffId, setSelectedStaffId] = useState<string | null>(null);
  const [selectedStaffName, setSelectedStaffName] = useState("");

  const [showNewStaffForm, setShowNewStaffForm] = useState(false);
  const [newStaffName, setNewStaffName] = useState("");
  const [newStaffExternalId, setNewStaffExternalId] = useState("");
  const [newStaffImageUrl, setNewStaffImageUrl] = useState("");

  const [role, setRole] = useState("");
  const [characterName, setCharacterName] = useState("");
  const [isSubmitting, setSubmitting] = useState(false);

  function selectStaff(staffId: string, staffName: string) {
    setSelectedStaffId(staffId);
    setSelectedStaffName(staffName);
    setShowNewStaffForm(false);
  }

  async function handleSubmit() {
    if (!role.trim()) {
      toast.error("役割を入力してください。");
      return;
    }

    setSubmitting(true);
    try {
      let staffId = selectedStaffId;
      if (showNewStaffForm || !staffId) {
        if (!newStaffName.trim()) {
          toast.error("氏名を入力するか、既存スタッフを選択してください。");
          setSubmitting(false);
          return;
        }
        const staff = await createStaff({ name: newStaffName, externalId: newStaffExternalId, imageUrl: newStaffImageUrl });
        staffId = staff.id;
      }

      await onLink(staffId!, role, characterName || undefined);
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "スタッフの追加に失敗しました。");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal open={open} onClose={onClose} title="スタッフを追加" maxWidth={560}>
      <div className="search-box" style={{ marginBottom: 14 }}>
        🔍 <input
          type="text"
          placeholder="氏名で検索…"
          value={name}
          onChange={(event) => setName(event.target.value)}
        />
      </div>

      {isLoading ? <p>検索中です...</p> : null}
      {name.trim().length > 0 && !isLoading && results.length === 0 ? <p>該当するスタッフが見つかりませんでした。</p> : null}
      {results.map((staff) => (
        <div key={staff.id} className="result-row">
          <div className="thumb" style={{ borderRadius: 999, width: 44, height: 44 }} />
          <div className="info">
            <div className="title">{staff.name}</div>
            <div className="sub">紐付け作品 {staff.linked_item_count}件</div>
          </div>
          <button
            type="button"
            className={selectedStaffId === staff.id ? "btn btn-sm" : "btn btn-accent btn-sm"}
            disabled={selectedStaffId === staff.id}
            onClick={() => selectStaff(staff.id, staff.name)}
          >
            {selectedStaffId === staff.id ? "選択済み" : "選択"}
          </button>
        </div>
      ))}

      <hr className="rail-divider" />

      <button type="button" className="tag-add-trigger" style={{ marginBottom: 12 }} onClick={() => setShowNewStaffForm((value) => !value)}>
        + 新規スタッフを登録
      </button>
      {showNewStaffForm ? (
        <div className="form-grid" style={{ marginBottom: 18 }}>
          <div className="form-field full">
            <label>
              氏名 <span className="required">*</span>
            </label>
            <input type="text" placeholder="例: 新津 明日香" value={newStaffName} onChange={(event) => setNewStaffName(event.target.value)} />
          </div>
          <div className="form-field">
            <label>external_id</label>
            <input type="text" placeholder="任意" value={newStaffExternalId} onChange={(event) => setNewStaffExternalId(event.target.value)} />
          </div>
          <div className="form-field">
            <label>image_url</label>
            <input type="text" placeholder="任意" value={newStaffImageUrl} onChange={(event) => setNewStaffImageUrl(event.target.value)} />
          </div>
        </div>
      ) : null}

      <hr className="rail-divider" />

      <div className="form-grid">
        <div className="form-field">
          <label>
            役割 <span className="required">*</span>
          </label>
          <input type="text" placeholder="例: 監督 / 脚本 / 声優" value={role} onChange={(event) => setRole(event.target.value)} />
        </div>
        <div className="form-field">
          <label>キャラ名</label>
          <input type="text" placeholder="声優の場合のみ(任意)" value={characterName} onChange={(event) => setCharacterName(event.target.value)} />
        </div>
      </div>

      {selectedStaffId && !showNewStaffForm ? <p style={{ marginTop: 10, fontSize: 12 }}>選択中: {selectedStaffName}</p> : null}

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
