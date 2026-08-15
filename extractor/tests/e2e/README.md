# Worker E2E tests

The suite uses the real PostgreSQL, API, worker, filesystem, PDF parser, and CPU OCR engine.

```powershell
$env:INTERNAL_API_KEY = "local-e2e-only-key"
$env:MEDIAVAULT_DB_PASSWORD = "local-e2e-db-password"
docker compose -f docker-compose.yml -f docker-compose.e2e.yml up -d --build mediavault-postgres mediavault-api mediavault-extractor
$env:MEDIAVAULT_E2E = "1"
cd extractor
uv run pytest -m e2e -v --junitxml=e2e-results.xml
```

Run `-m "e2e and not slow"` to exclude real OCR. JUnit properties named
`ocr_total_seconds` and `ocr_seconds_per_page` contain measurements for lease and timeout tuning.

Before sharing logs, verify that neither the value of `INTERNAL_API_KEY` nor fixture text appears:

```powershell
docker compose logs mediavault-extractor
```
