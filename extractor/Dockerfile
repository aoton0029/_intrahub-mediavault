FROM ghcr.io/astral-sh/uv:0.8.4 AS uv

FROM python:3.12-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    UV_COMPILE_BYTECODE=1 \
    UV_LINK_MODE=copy \
    HF_HOME=/var/cache/mediavault-extractor/huggingface

COPY --from=uv /uv /uvx /bin/
WORKDIR /app

COPY pyproject.toml uv.lock ./
RUN uv sync --frozen --no-dev --no-install-project

COPY src ./src
RUN uv sync --frozen --no-dev

# Yomitoku models are intentionally fetched lazily on first OCR use and cached in
# HF_HOME. Baking the large, device-dependent model into this foundation image is
# deferred until production measurements settle the model/version in prep.md.
CMD ["uv", "run", "--no-sync", "python", "-m", "mediavault_extractor"]

