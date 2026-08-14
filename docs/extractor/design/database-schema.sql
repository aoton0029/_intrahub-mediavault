-- ========================================
-- MediaVault Extractor データベーススキーマ
-- ========================================
--
-- 作成日: 2026-08-14
-- 関連設計: architecture.md
-- 関連要件: ../spec/requirements.md REQ-040〜044, REQ-103, REQ-206, REQ-409
--
-- 適用方法:
--   backend/mediavault-api/migrations/{timestamp}_add_extraction.up.sql として追加する。
--   既存の 20260623000001_init_schema.up.sql は改変しない。
--
-- 信頼性レベル:
-- - 🔵 青信号: EARS要件定義書・設計文書・既存DBスキーマを参考にした確実な定義
-- - 🟡 黄信号: EARS要件定義書・設計文書・既存DBスキーマから妥当な推測による定義
-- - 🔴 赤信号: EARS要件定義書・設計文書・既存DBスキーマにない推測による定義
--
-- 既存スキーマとの整合方針:
--   * id は UUID PRIMARY KEY DEFAULT gen_random_uuid()（init_schema 全テーブル共通）
--   * created_at / updated_at は TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
--     （TIMESTAMPTZ ではない。init_schema:52 等に合わせる）
--   * updated_at は init_schema:240 で定義済みの update_updated_at_column() トリガーで更新する
--   * ENUM は CREATE TYPE で定義し、sqlx::Type で Rust 側へマップする（file_type と同方式）
--

-- ========================================
-- ENUM 定義
-- ========================================

-- 抽出の状態
-- 🔵 信頼性: requirements.md §状態遷移・REQ-201〜203・PRD §8.5 に直接対応
-- 終端状態: succeeded / failed / cancelled
CREATE TYPE extraction_state AS ENUM (
    'queued',
    'running',
    'cancelling',
    'succeeded',
    'failed',
    'cancelled'
);

-- ファイル参照のルート種別
-- 🔵 信頼性: 設計ヒアリングQ4（root 種別 + 相対パス）・architecture.md D-3 に直接対応
-- item_files.path の2経路（リンク=絶対パス / アップロード=STORAGE_ROOT 相対）を吸収するため、
-- worker へはマウントパスに依存しない root 種別で渡す。
-- 本 ENUM は DB カラムではなく API レスポンスの型として使うため、CREATE TYPE はしない。
-- （Rust 側 models::item_extraction::FileRefRoot として定義する。interfaces.rs 参照）

-- ========================================
-- テーブル定義
-- ========================================

-- 抽出ジョブ（1ファイルに対する1回の抽出試行）
-- 🔵 信頼性: requirements.md REQ-040・PRD §8.1 から job_type / dedup_key / target_item_id を
--            除去して再構成したもの
--
-- 設計判断（architecture.md D-2）: 履歴を残す。再抽出のたびに新しい行を追加し、
-- GET .../extraction は created_at DESC の最新1件を返す。
CREATE TABLE item_file_extractions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 🔵 init_schema 全テーブル共通パターン

    -- 対象ファイル。ファイル削除時は抽出履歴も一緒に消す
    -- 🔵 REQ-409（Item/ファイル削除時の一貫性）・init_schema:124 の item_files → items と同方針
    item_file_id UUID NOT NULL REFERENCES item_files(id) ON DELETE CASCADE,

    state extraction_state NOT NULL DEFAULT 'queued', -- 🔵 REQ-040・REQ-201

    -- 試行回数と上限。fail 時に判定する
    -- 🟡 max_attempts の既定値 3 は prep.md §確認事項で判断待ち。CPU OCR が長時間かかることを
    --    踏まえ、大きすぎると壊れたファイルで無駄なCPUを消費するため小さめに置く
    attempts INTEGER NOT NULL DEFAULT 0, -- 🔵 REQ-040・REQ-111/112
    max_attempts INTEGER NOT NULL DEFAULT 3, -- 🟡 prep.md 判断待ち

    -- 進捗（ページ単位）。heartbeat で更新する
    -- 🔵 REQ-023・REQ-066・PRD §8.1
    progress_current INTEGER NOT NULL DEFAULT 0,
    progress_total INTEGER, -- NULL = 総数未確定（ファイルを開く前）

    -- worker の排他取得と回収
    -- 🔵 REQ-021・REQ-118・PRD §8.1
    claimed_by VARCHAR(255),        -- worker 識別子（hostname 等）。観測用
    lease_token UUID,               -- claim 時に発行。complete/fail/cancelled で照合（REQ-407）
    lease_expires_at TIMESTAMP,     -- 経過後に再claim可能（REQ-118・NFR-202）

    -- 構造化エラー。{ "kind": "...", "message": "...", "retryable": bool, ... }
    -- 🔵 REQ-026・REQ-301・NFR-503
    error JSONB,

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 🔵 共通パターン
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 🔵 共通パターン

    -- 進捗は非負
    -- 🟡 データ健全性のための妥当な追加（要件に明記はない）
    CONSTRAINT chk_item_file_extractions_progress
        CHECK (progress_current >= 0 AND (progress_total IS NULL OR progress_total >= 0)),

    -- 試行回数は非負かつ上限以下
    -- 🟡 REQ-111/112 の判定を DB 側でも壊れないよう担保
    CONSTRAINT chk_item_file_extractions_attempts
        CHECK (attempts >= 0 AND attempts <= max_attempts),

    -- running / cancelling では必ず lease が存在する。逆に終端状態では lease を持たない
    -- 🟡 REQ-407・EDGE-002 の前提を DB 側で壊れないよう担保する妥当な追加
    CONSTRAINT chk_item_file_extractions_lease
        CHECK (
            (state IN ('running', 'cancelling')
                AND lease_token IS NOT NULL AND lease_expires_at IS NOT NULL)
            OR state NOT IN ('running', 'cancelling')
        )
);

-- 【最重要】1ファイルにつき未完了の抽出は最大1件
-- 🔵 信頼性: REQ-044・ヒアリングQ1（要件定義フェーズ）・architecture.md D-1 に直接対応
--
-- PRD §8.1 が要求していた dedup_key による重複防止を、この部分UNIQUE index が置き換える。
-- 冪等な POST .../extraction（REQ-101）はこの制約に依存する:
--   INSERT を試み、23505（unique_violation）なら既存の未完了行を SELECT して 200 で返す。
-- 終端状態（succeeded / failed / cancelled）の行は制約対象外のため、履歴として複数残せる。
CREATE UNIQUE INDEX uq_item_file_extractions_active
    ON item_file_extractions (item_file_id)
    WHERE state IN ('queued', 'running', 'cancelling');

-- 抽出結果（現行の1件のみ。履歴は持たない）
-- 🔵 信頼性: requirements.md REQ-041〜043・PRD §8.2・item-text.md §データモデルへの要求に直接対応
CREATE TABLE item_file_texts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- 🔵 共通パターン

    -- 同一ファイルの現行結果は1件のみ。再抽出は UPSERT で置き換える（REQ-103）
    -- 🔵 item-text.md「item_files への FK・UNIQUE」に直接対応
    item_file_id UUID NOT NULL UNIQUE REFERENCES item_files(id) ON DELETE CASCADE,

    -- 正規化済み全文。分割せず全文で保持し、チャンク分割は読み出し時に行う
    -- 🔵 item-text.md「chunk_size をクエリで変えられるようにするため」に直接対応
    content TEXT NOT NULL,

    -- ページ・章の文字範囲と表示ラベル
    -- [{"start": 0, "end": 1200, "label": "p.1"}, ...]
    -- start は含む / end は含まない（half-open）。start/end は文字オフセット（バイトではない）
    -- 🔵 REQ-042・ヒアリングQ5（要件定義フェーズ）・architecture.md D-5 に直接対応
    boundaries JSONB NOT NULL DEFAULT '[]'::jsonb,

    -- 抽出ロジックと境界の版。変わると保存済み chunk_index が別箇所を指す
    -- 🔵 REQ-104・item-text.md §extraction_version に直接対応
    extraction_version VARCHAR(64) NOT NULL,

    -- 使用方式とエンジン情報
    -- {"method": "mixed", "embedded_text_pages": 7, "ocr_pages": 3,
    --  "ocr": {"engine": "yomitoku", "device": "cpu", "model": "..."}}
    -- 🔵 REQ-043・PRD FR-007 / architecture.md D-8（記録粒度は 🟡）
    extractor JSONB NOT NULL,

    extracted_at TIMESTAMP NOT NULL, -- 🔵 REQ-041・PRD §8.2

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 🔵 共通パターン
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 🔵 共通パターン

    -- boundaries は JSON 配列であること
    -- 🟡 EDGE-107（boundaries の整合性検証）の一部を DB 側で担保。
    --    start <= end および end <= char_length(content) の検証はアプリ側で行う
    --    （jsonb_array_elements を使う CHECK は不可のため）
    CONSTRAINT chk_item_file_texts_boundaries_is_array
        CHECK (jsonb_typeof(boundaries) = 'array'),

    -- extractor は JSON オブジェクトであること
    -- 🟡 同上
    CONSTRAINT chk_item_file_texts_extractor_is_object
        CHECK (jsonb_typeof(extractor) = 'object')
);

-- ========================================
-- インデックス
-- ========================================

-- GET .../extraction の「最新1件」取得（architecture.md D-2）
-- 🔵 信頼性: REQ-002・architecture.md D-2 に直接対応
CREATE INDEX idx_item_file_extractions_file_created
    ON item_file_extractions (item_file_id, created_at DESC);

-- claim の対象行探索（state='queued' または lease 切れの running）
-- 部分indexにすることで、終端状態の履歴行が増えても claim の走査量が増えない
-- 🔵 信頼性: REQ-020・REQ-118・EDGE-001（FOR UPDATE SKIP LOCKED）に直接対応
CREATE INDEX idx_item_file_extractions_claimable
    ON item_file_extractions (created_at)
    WHERE state IN ('queued', 'running', 'cancelling');

-- 運用メトリクス（待機数・成功率・lease切れ回数）の集計用
-- 🟡 信頼性: NFR-402（観測可能にする）から妥当な推測
CREATE INDEX idx_item_file_extractions_state
    ON item_file_extractions (state);

-- item_file_texts.item_file_id は UNIQUE 制約により自動で index が張られるため、
-- 追加の index は不要。
-- 🔵 信頼性: PostgreSQL の UNIQUE 制約仕様より

-- ========================================
-- トリガー
-- ========================================

-- updated_at 自動更新
-- update_updated_at_column() は init_schema:240 で定義済みのため再定義しない
-- 🔵 信頼性: 既存 init_schema の共通パターンに直接対応
CREATE TRIGGER update_item_file_extractions_updated_at
    BEFORE UPDATE ON item_file_extractions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_item_file_texts_updated_at
    BEFORE UPDATE ON item_file_texts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ========================================
-- 主要クエリ（実装時の参照用）
-- ========================================

-- --- claim: 実行可能な抽出を1件だけ排他的に取得する ---
-- 🔵 信頼性: REQ-020・REQ-021・REQ-118・EDGE-001・TC-020-B01/B02 に直接対応
--
-- SKIP LOCKED により、2台の worker が同時に実行してもブロックせず別々の行を取る。
-- lease 切れの running / cancelling も対象に含めることで、worker 異常終了からの復旧を兼ねる。
--
-- WITH claimable AS (
--     SELECT id FROM item_file_extractions
--     WHERE state = 'queued'
--        OR (state IN ('running', 'cancelling') AND lease_expires_at < CURRENT_TIMESTAMP)
--     ORDER BY created_at
--     FOR UPDATE SKIP LOCKED
--     LIMIT 1
-- )
-- UPDATE item_file_extractions e
-- SET state = 'running',
--     attempts = e.attempts + 1,
--     claimed_by = $1,
--     lease_token = gen_random_uuid(),
--     lease_expires_at = CURRENT_TIMESTAMP + ($2 || ' seconds')::interval
-- FROM claimable c
-- WHERE e.id = c.id
-- RETURNING e.*;
--
-- 注意: attempts <= max_attempts の CHECK があるため、attempts が上限に達した行を
--       再claimしようとすると制約違反になる。claim の WHERE に
--       `AND attempts < max_attempts` を含め、上限到達行は claim 対象から外す。
--       上限到達かつ lease 切れの行は、別途 sweeper で failed へ遷移させる（下記）。

-- --- lease 切れかつ試行上限に達した行を failed へ落とす（sweeper） ---
-- 🟡 信頼性: REQ-111・NFR-202 から妥当な推測。claim 時に併せて実行するか、
--            定期実行するかは実装フェーズで決める
--
-- UPDATE item_file_extractions
-- SET state = 'failed',
--     lease_token = NULL,
--     lease_expires_at = NULL,
--     error = jsonb_build_object(
--         'kind', 'lease_expired',
--         'message', 'workerが応答しないまま試行上限に達しました',
--         'retryable', false
--     )
-- WHERE state IN ('running', 'cancelling')
--   AND lease_expires_at < CURRENT_TIMESTAMP
--   AND attempts >= max_attempts;

-- --- complete: 抽出結果保存と成功遷移を同一トランザクションで確定する ---
-- 🔵 信頼性: REQ-024・REQ-025・TC-024-01・PRD §8.8「抽出結果保存とジョブ成功が不整合にならない」
--
-- BEGIN;
--   -- 1) lease token 照合 + 状態確認（行ロック）
--   SELECT item_file_id, state FROM item_file_extractions
--   WHERE id = $1 AND lease_token = $2 AND state = 'running'
--   FOR UPDATE;
--   -- 0行なら INVALID_LEASE_TOKEN(409)。state='cancelling' の場合もここで弾かれる（REQ-204）
--
--   -- 2) 抽出結果を UPSERT（再抽出時は置き換え。REQ-103）
--   INSERT INTO item_file_texts
--       (item_file_id, content, boundaries, extraction_version, extractor, extracted_at)
--   VALUES ($3, $4, $5, $6, $7, $8)
--   ON CONFLICT (item_file_id) DO UPDATE SET
--       content = EXCLUDED.content,
--       boundaries = EXCLUDED.boundaries,
--       extraction_version = EXCLUDED.extraction_version,
--       extractor = EXCLUDED.extractor,
--       extracted_at = EXCLUDED.extracted_at;
--
--   -- 3) 抽出を succeeded へ
--   UPDATE item_file_extractions
--   SET state = 'succeeded', lease_token = NULL, lease_expires_at = NULL,
--       progress_current = progress_total
--   WHERE id = $1;
-- COMMIT;

-- --- Item Text API: チャンク切り出し（全文をアプリメモリへ載せない） ---
-- 🔵 信頼性: REQ-008・NFR-001・EDGE-103・item-text.md §実装上の注意に直接対応
--
-- total_chunks は CHAR_LENGTH（文字数）で算出する。OCTET_LENGTH（バイト長）では
-- 日本語テキストで境界がずれる。
--
-- SELECT
--     t.extraction_version,
--     t.extracted_at,
--     t.boundaries,
--     CEIL(CHAR_LENGTH(t.content)::numeric / $2)::int AS total_chunks,
--     SUBSTRING(t.content FROM ($1 * $2 + 1) FOR $2) AS chunk_text
-- FROM item_file_texts t
-- WHERE t.item_file_id = $3;
--   -- $1 = chunk_index (0起点), $2 = chunk_size, $3 = item_file_id
--   -- SUBSTRING の FROM は1起点のため +1 する

-- --- 主ファイルの解決（file_id 省略時） ---
-- 🔵 信頼性: REQ-115・item-text.md §主ファイルの解決・TC-005-E03 に直接対応
--
-- 抽出済みファイルを候補とし、1件なら採用、0件なら FILE_NOT_FOUND か TEXT_NOT_EXTRACTED、
-- 2件以上なら AMBIGUOUS_FILE（候補一覧付き）。推測で選ばない。
--
-- SELECT f.id, f.label, f.file_type
-- FROM item_files f
-- INNER JOIN item_file_texts t ON t.item_file_id = f.id
-- WHERE f.item_id = $1
-- ORDER BY f.created_at;

-- --- 運用メトリクス（NFR-402） ---
-- 🟡 信頼性: NFR-402 から妥当な推測
--
-- SELECT
--     state,
--     COUNT(*) AS count,
--     AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) AS avg_seconds
-- FROM item_file_extractions
-- GROUP BY state;

-- ========================================
-- ロールバック（.down.sql）
-- ========================================
--
-- DROP TRIGGER IF EXISTS update_item_file_texts_updated_at ON item_file_texts;
-- DROP TRIGGER IF EXISTS update_item_file_extractions_updated_at ON item_file_extractions;
-- DROP TABLE IF EXISTS item_file_texts;
-- DROP TABLE IF EXISTS item_file_extractions;
-- DROP TYPE IF EXISTS extraction_state;
--
-- update_updated_at_column() は init_schema 所有のため DROP しない。
-- 🔵 信頼性: 既存 init_schema の所有関係より

-- ========================================
-- 初期データ
-- ========================================
--
-- 投入するマスターデータはない。抽出はすべてユーザー/AIエージェントの明示的な
-- リクエストで作られる（REQ-401: ファイル登録時の自動キューは行わない）。
-- 🔵 信頼性: ヒアリングQ6（要件定義フェーズ）・REQ-401 に直接対応

-- ========================================
-- 信頼性レベルサマリー
-- ========================================
-- 定義項目（テーブル2 / ENUM1 / カラム21 / 制約5 / index4 / トリガー2 / 主要クエリ6）:
-- - 🔵 青信号: 32件 (78%)
-- - 🟡 黄信号: 9件 (22%)
-- - 🔴 赤信号: 0件 (0%)
--
-- 品質評価: ✅ 高品質
--
-- 🟡 の内訳:
--   * max_attempts の既定値 3（prep.md §確認事項で判断待ち）
--   * データ健全性のための CHECK 制約 4件（要件に明記はないが破壊を防ぐ妥当な追加）
--   * 運用メトリクス用 index とクエリ（NFR-402 からの具体化）
--   * lease 切れ sweeper の実行タイミング（実装フェーズで決定）
