← [index](./index.md)

# Collection API

## GET /collection/overview
コレクション全体の統計を1回で返す集計エンドポイント。`GET /items` が総件数を返さない設計のため、status別・お気に入り別の件数はこのAPIでのみ取得できる。

- **認証**: 不要
- **クエリパラメータ**:
  - `recent_limit` (u32, optional, default `10`, 範囲 `1..=50`) — `recently_added` / `recently_updated` の返却件数。範囲外（`0`または`51`以上）は400 `VALIDATION_ERROR`
- **成功レスポンス** (200): `ApiOk<CollectionOverview>`

```json
{
  "success": true,
  "data": {
    "total_items": 181,
    "favorite_count": 34,
    "by_media_type": [
      { "key": "anime", "count": 42 },
      { "key": "movie", "count": 10 },
      { "key": "drama", "count": 3 },
      { "key": "manga", "count": 87 },
      { "key": "novel", "count": 15 },
      { "key": "game", "count": 21 },
      { "key": "academic_book", "count": 2 },
      { "key": "paper", "count": 1 }
    ],
    "by_status": [
      { "key": "not_started", "count": 90 },
      { "key": "in_progress", "count": 40 },
      { "key": "completed", "count": 51 }
    ],
    "recently_added": [ /* ItemWithRefs[]（GET /items と同一形状） */ ],
    "recently_updated": [ /* ItemWithRefs[] */ ]
  }
}
```

- `by_media_type` / `by_status` は該当0件の種別・ステータスも `count: 0` のエントリとして含む（コレクションが空の場合も配列自体は8件/3件のまま、`count`が全て0になる）
- `recently_added` は `created_at` 降順、`recently_updated` は `updated_at` 降順で、それぞれ独立に取得した上位`recent_limit`件
- コレクションが0件の場合もエラーにせず200を返す（`total_items: 0`、`recently_added`/`recently_updated`は空配列）

### 既存 `GET /items/counts-by-media-type` との関係
既存エンドポイントはフロントエンドのサイドバーが利用しているため削除・変更しない。`by_media_type`の集計ロジックは`item_repository::count_items_by_media_type`を共通利用しており、両エンドポイントで件数が完全に一致する。
