# 20_movie_detail 未決事項

AskUserQuestionで質問したが結論が出ず保留にした事項、およびタスク実装中にCodexが仮決定した事項を記録する（設計書起因の【要確認】項目は原則ステップ5でAskUserQuestionにより解消済みのため、ここには残らない）。

## 未決事項

（現時点でなし）

## ユーザーによる決定事項

- [x] `/media/:id` の振り分け方針: `media_type`に応じてディスパッチする`MediaDetailPage`を新設し、`AnimeDetailPage`/`MovieDetailPage`を振り分ける（[16_anime_detail/02_open_questions.md](../16_anime_detail/02_open_questions.md)からの継続決定）
- [x] 「編集する」ボタンの遷移先: 一般メディア編集フォームは未実装のため、既存フォームへのリンクとしてパスを仮定して実装するに留める（フォーム自体の実装は本タスク範囲外）
- [x] 概要セクションの出典: アニメ詳細と同様 `Item.description` を使う

## Codexによる仮決定ログ

- [x] 「編集する」ボタンは `/media/:id/edit` へのリンクのみ実装し、編集フォーム本体は未実装のままとした
