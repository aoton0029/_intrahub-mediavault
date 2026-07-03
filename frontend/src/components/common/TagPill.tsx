interface TagPillProps {
  label: string
}

/**
 * 【機能概要】: タグ名を受け取り、#プレフィックス付きのハッシュタグ風表示で描画する
 * 【設計方針】: _shared.css .tag-pill 相当のスタイルを適用し、テスト容易性のため #label をJSX内で連結する
 * 🔵 信頼性レベル: architecture.md「共通コンポーネント層」・_shared.css .tag-pill定義より
 */
export function TagPill({ label }: TagPillProps) {
  return (
    <span className="tag-pill" data-testid="tag-pill">
      #{label}
    </span>
  )
}
