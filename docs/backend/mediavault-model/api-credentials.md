# api_credentials

`backend/mediavault-api/migrations/20260623000002_add_relation_tables.up.sql` / `backend/mediavault-api/src/models/api_credential.rs`

## DBスキーマ

### api_credentials

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| provider | api_provider PK | NOT NULL | |
| api_key | VARCHAR(500) | NOT NULL | |
| updated_at | TIMESTAMP | NOT NULL | DEFAULT CURRENT_TIMESTAMP、トリガー`trg_api_credentials_updated_at`で自動更新 |

他テーブルとのFK関係はない独立テーブル。

## Rustモデル

```rust
#[sqlx(type_name = "api_provider")]
#[serde(rename_all = "snake_case")]
pub enum ApiProvider {
    Tmdb, Igdb, Ndl, Steam,
    #[sqlx(rename = "openlibrary")] OpenLibrary,
    #[sqlx(rename = "anilist")] AniList,
    // Jikanはキー不要のため対象外
}
```

DB上のENUM値は`openlibrary`/`anilist`（アンダースコアなし）だが、API/Rust側のserde表現は`rename_all = "snake_case"`デフォルトにより`open_library`/`ani_list`となる。`OpenLibrary`/`AniList`のみ`#[sqlx(rename = ...)]`でDB側実値に個別マッピングし、API文字列とDB格納値の対応を分離している。

- `ApiCredential { provider: ApiProvider, api_key: String, updated_at: NaiveDateTime }`（`sqlx::FromRow`）
- `UpdateApiKeyRequest { api_key: String }` — `PUT /settings/api-keys/:provider`のリクエストボディ。
- `parse_api_provider(raw: &str) -> Option<ApiProvider>` — パスパラメータ文字列`tmdb`/`igdb`/`ndl`/`steam`/`open_library`/`ani_list`のみを許可するmatch文。大文字・別表記（`TMDB`, `Tmdb`, 末尾空白）や`jikan`は`None`（`INVALID_PROVIDER`エラーの元になる）。

## 参照

エンドポイント例は [mediavault-api/settings.md](../mediavault-api/settings.md) を参照。
