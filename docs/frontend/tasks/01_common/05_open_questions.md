# 05. 未確定事項（要確認）の集約

実装時に発見した要確認事項はここに追記していく運用とする。各画面設計書側にも個別の【要確認】がある可能性があるため、実装着手時に該当設計書を確認し、見つかったものは随時ここへ追加すること。

## 共通設計（00_common.md）由来

- [ ] **ライトモードの実現方式**: `prefers-color-scheme` ではなく `data-theme` 属性による明示的トグルで実現する方針。Tailwindの `dark:` バリアントは使わず `[data-theme="light"]` セレクタベースのCSSに寄せる（[00_common.md §2](../design/00_common.md#2-tailwind-v4-theme-トークン対応表)）
- [ ] **解除（マイリスト/関連作品）アイコン**: モックSVGは箱アイコンのため、実装時に `FiPackage`/`FiX` 等、モックのpath形状に近いものを選定する必要あり（[00_common.md §4](../design/00_common.md#4-アイコンreact-icons)）
- [ ] **並び替えアイコン**: モックはカスタムpathで `react-icons` に厳密一致するものが無い場合、最も近いソート系アイコンを選定する（[00_common.md §4](../design/00_common.md#4-アイコンreact-icons)）
- [ ] **API仕様の未確定箇所**: `docs/frontend/PRD.md` の「バックエンドAPI」節は現状未記載。各画面設計書の「API連携」章の記載はPRDの機能一覧からの推測であり、実装時に `docs/backend/mediavault-api/*.md` と突き合わせて確定させる必要がある（[00_common.md §7](../design/00_common.md#7-api連携についての注記)）

## 画面別設計書由来

（実装着手時に各設計書を確認し、見つかった項目をここに追記する）
