# 19_settings 実装タスク概要

`docs/frontend/design/19_settings.md`（以下「設計書」）を実装に落とし込むための計画。対象は設定画面（`/settings`）で、API連携キー管理・データインポート・システム状態確認の3タブを持つ単一画面。既存の共通コンポーネント`SettingsShell`（タブ切替）・`ApiKeyCard`（プロバイダ行カード）を再利用・拡張し、`ApiKeysPanel`/`ImportPanel`/`SystemStatusPanel`を新規実装する。

## 決定事項（AskUserQuestionで確認済み）

- **APIプロバイダ一覧**: バックエンド仕様（`docs/backend/mediavault-api/settings.md`）を正とする。`PUT /settings/api-keys/{provider}`の対象は `tmdb`, `igdb`, `ndl`, `steam`, `annict`, `rakuten` の6種（パスワード入力欄+保存ボタンを表示）。`jikan`はAPI対象外だが設計書通り「設定不要」の読み取り専用行として表示する。設計書にある`open_library`/`ani_list`は実装しない。
- **ApiKeyCardの拡張方針**: 既存の`frontend/src/components/shared/ApiKeyCard.tsx`（`provider`/`keyMasked`/`onEdit`）を拡張する。設定画面向けに、パスワード入力欄+保存ボタンをインライン表示するモード（例: `onSave(value: string)`と`variant`または新規propsの追加）を追加し、既存の呼び出し箇所（あれば）への影響が出ないようにする。
- **タブ管理方式**: 既存の`SettingsShell`（内部`useState`でタブ管理、URLとは非連動）をそのまま使う。URLクエリパラメータ/ネストルートとの連動は行わない。

## タスクファイル構成

| ファイル | 内容 | 対応する設計書セクション |
|---|---|---|
| [01_settings_screen.md](01_settings_screen.md) | `SettingsPage`（`/settings`）本体・3パネル・`ApiKeyCard`拡張・ルート差し替えの実装 | §1〜§7 |
| [02_open_questions.md](02_open_questions.md) | 設計書中の【要確認】項目・実装中の仮決定事項の追跡 | §6 |

## Claude ⇄ Codex 運用ルール

このタスク群は Claude が計画・レビューを行い、実装は Codex に委譲する前提で書式を統一している。

- 各タスクファイルの「タスク一覧」「テストリスト」のチェックボックス `- [ ]` は、実装・テストが完了するたびに **Codexが** `- [x]` に変更する（Claude側では変更しない）。
- タスクを完了する際、対応するタスク見出しの直下にある `> Codexメモ:` 行に、実装上の判断・設計書との差分・未解決事項を1〜3行で追記する（無ければ `> Codexメモ: (なし)` のままでよい）。
- 【要確認】に該当する意思決定が必要になった場合は、実装を進めず [02_open_questions.md](02_open_questions.md) に追記し、Claudeのレビューを待つ。ブロッキングでない場合は妥当な仮決定を行い、その旨をメモに残して先に進んでよい。
- **実装範囲の限定**: 着手するタスクの「タスク一覧」に書かれた内容の実装のみを行う。参照してよいファイルは各タスクファイルの「前提ファイル」節に列挙されたものと、そこから直接importされるファイルに限定する。それ以外のファイルを探すための横断的なコードベース探索（grep/find等での関連ファイル探索、他タスクの実装状況の確認等）は行わない。前提ファイルの範囲で判断がつかない場合は、推測で探索を広げず [02_open_questions.md](02_open_questions.md) に記載するか、タスク側で仮決定してCodexメモに記載する。

## 完了の定義（DoD）

各タスクは以下をすべて満たして完了とする。

- [ ] `yarn lint` が通る
- [ ] `yarn test` が通る（対象タスクのユニットテストを含む）
- [ ] `yarn build`（`tsc -b && vite build`）が型エラーなく通る
- [ ] 設計書の該当セクションで定義されたprops/クラス名/挙動と実装が一致している（差異があればCodexメモに記載）
- [ ] `docs/frontend/ui/19_settings.html` + `_shared.css` の見た目・DOM構造とTailwind実装がセマンティックに一致している
- [ ] `yarn test:e2e` の実装⇔モック レイアウト一致テストが通る
