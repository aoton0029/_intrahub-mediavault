# TASK-0023 E2E・CPU OCR計測結果

計測日: 2026-08-15  
構成: Docker Desktop / CPU / Python 3.12 / yomitoku 0.14.0  
モデル: `dbnetv2_1+parseq-small`（CPU既定の軽量モデル）

## E2E結果

- テキストレイヤーPDF: 成功。本文、ページラベル、`embedded_text`、再抽出、再現性を確認
- スキャンPDF: 成功。3ページすべてCPU OCR
- 混在PDF: 成功。埋め込み2ページ、OCR 1ページ
- PNG画像: 成功。CPU OCR
- 破損PDF: `corrupt_file` / retryable=false / attempts=1
- PNGをPDF拡張子にしたファイル: `unsupported_format` / retryable=false
- 処理中キャンセル: `running -> cancelling -> cancelled`、本文未保存
- workerコンテナ起動: OpenCVランタイム依存をDockerfileへ追加後に成功
- 構造化完了ログ: 必須9項目を実ログで確認

## CPU OCR計測

| fixture | OCRページ数 | API要求から完了 | 1 OCRページあたり |
|---|---:|---:|---:|
| `scanned.pdf` | 3 | 2.892秒 | 0.964秒 |
| `mixed.pdf` | 1 | 0.842秒 | 0.842秒 |
| `japanese.png` | 1 | 1.051秒 | 1.051秒 |

worker自身の構造化ログでは、それぞれ2.647秒、0.660秒、0.868秒だった。上表はAPIポーリングを含む利用者視点の値である。

## 暫定値の評価

- 本fixtureでは lease 5秒・heartbeat 1秒で正常完了した。
- 本番既定の lease 300秒・heartbeat 30秒は、今回の軽量fixtureに対して十分な余裕がある。
- `EXTRACTOR_JOB_TIMEOUT_SEC=3600` の確定には、リポジトリへ含めない実運用最大PDFでの追加計測が必要。
- 通常モデルとの速度・精度比較は、現在のCPU実装が軽量モデル固定のため未計測。実運用データと通常モデルを使うDIRECT作業として残る。
- OCRフォールバック閾値50は、3ページfixtureの期待分類（text=3/0、mixed=2/1、scanned=0/3）と一致した。実データ分布での追加評価は必要。

計測値は `pytest -m "e2e and slow" --junitxml=e2e-results.xml` の testcase properties にも出力される。
