//! `item_files.path` をworker向けの安全なファイル参照へ変換する。
//!
//! 本サービスは読み取り用の参照解決と検証のみを担う。アップロードの保存・削除は
//! `file_storage` の責務であり、ここからファイルシステムを書き換えることはない。

use std::path::{Component, Path, PathBuf};

use crate::models::item_extraction::{FileRef, FileRefRoot};
use crate::models::response::{ApiError, ApiErrorCode};

const STORAGE_ROOT_ENV: &str = "STORAGE_ROOT";
const STORAGE_SUBDIR_FILES_ENV: &str = "STORAGE_SUBDIR_FILES";
const LIBRARY_ROOT_ENV: &str = "LIBRARY_ROOT";
const DEFAULT_STORAGE_ROOT: &str = "/srv/mediavault";
const DEFAULT_STORAGE_SUBDIR_FILES: &str = "files";
const DEFAULT_LIBRARY_ROOT: &str = "/library";

#[derive(Debug, Clone)]
pub struct FileRefConfig {
    pub storage_root: PathBuf,
    pub storage_subdir_files: PathBuf,
    pub library_root: PathBuf,
}

impl FileRefConfig {
    pub fn from_env() -> Self {
        Self {
            storage_root: non_empty_env(STORAGE_ROOT_ENV)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(DEFAULT_STORAGE_ROOT)),
            storage_subdir_files: non_empty_env(STORAGE_SUBDIR_FILES_ENV)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(DEFAULT_STORAGE_SUBDIR_FILES)),
            library_root: non_empty_env(LIBRARY_ROOT_ENV)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(DEFAULT_LIBRARY_ROOT)),
        }
    }
}

impl Default for FileRefConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedFile {
    pub file_ref: FileRef,
    /// API内部で存在・可読性を確認した正規化済みパス。外部レスポンスへ含めない。
    pub absolute_path: PathBuf,
    pub size_bytes: i64,
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn validation_error() -> ApiError {
    ApiError::new(ApiErrorCode::ValidationError, "不正なファイルパスです")
}

fn unavailable_error() -> ApiError {
    ApiError::new(
        ApiErrorCode::UnprocessableEntity,
        "ファイルの実体を読み取れません",
    )
}

fn is_db_absolute(path: &str) -> bool {
    Path::new(path).is_absolute() || path.starts_with('/')
}

fn safe_relative_components(path: &Path) -> Result<Vec<String>, ApiError> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(validation_error());
    }

    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let part = part.to_str().ok_or_else(validation_error)?;
                if part.is_empty() {
                    return Err(validation_error());
                }
                parts.push(part.to_string());
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(validation_error());
            }
        }
    }

    if parts.is_empty() {
        Err(validation_error())
    } else {
        Ok(parts)
    }
}

fn relative_string(path: &Path) -> Result<String, ApiError> {
    Ok(safe_relative_components(path)?.join("/"))
}

fn build_reference<'a>(
    path: &str,
    config: &'a FileRefConfig,
) -> Result<(FileRef, PathBuf, &'a Path), ApiError> {
    if path.trim().is_empty() {
        return Err(validation_error());
    }

    let source = Path::new(path);
    if source
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(validation_error());
    }

    if is_db_absolute(path) {
        let relative = source
            .strip_prefix(&config.library_root)
            .map_err(|_| validation_error())?;
        let relative_path = relative_string(relative)?;
        Ok((
            FileRef {
                root: FileRefRoot::Library,
                relative_path,
            },
            source.to_path_buf(),
            &config.library_root,
        ))
    } else {
        let source_relative = relative_string(source)?;
        let subdir_relative = relative_string(&config.storage_subdir_files)?;
        let relative_path = format!("{subdir_relative}/{source_relative}");
        Ok((
            FileRef {
                root: FileRefRoot::Storage,
                relative_path,
            },
            config
                .storage_root
                .join(&config.storage_subdir_files)
                .join(source),
            &config.storage_root,
        ))
    }
}

/// DBに保存されたパスをworker向け参照へ変換し、実体・可読性・サイズを検証する。
pub fn resolve(path: &str, config: &FileRefConfig) -> Result<ResolvedFile, ApiError> {
    let (file_ref, candidate, allowed_root) = build_reference(path, config)?;

    let canonical_candidate = std::fs::canonicalize(&candidate).map_err(|err| {
        tracing::warn!(error = %err, "file_ref target is unavailable");
        unavailable_error()
    })?;
    let canonical_root = std::fs::canonicalize(allowed_root).map_err(|err| {
        tracing::error!(error = %err, "file_ref allowed root is unavailable");
        unavailable_error()
    })?;

    if !canonical_candidate.starts_with(&canonical_root) {
        tracing::warn!("file_ref target escaped its configured root");
        return Err(validation_error());
    }

    let metadata = std::fs::metadata(&canonical_candidate).map_err(|err| {
        tracing::warn!(error = %err, "file_ref metadata lookup failed");
        unavailable_error()
    })?;
    if !metadata.is_file() {
        return Err(unavailable_error());
    }

    // metadata取得だけでは権限不足を検出できないため、読み取り用に開けることも確認する。
    std::fs::File::open(&canonical_candidate).map_err(|err| {
        tracing::warn!(error = %err, "file_ref target is not readable");
        unavailable_error()
    })?;

    let size_bytes = i64::try_from(metadata.len()).map_err(|_| unavailable_error())?;
    Ok(ResolvedFile {
        file_ref,
        absolute_path: canonical_candidate,
        size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn config(storage_root: PathBuf, library_root: PathBuf) -> FileRefConfig {
        FileRefConfig {
            storage_root,
            storage_subdir_files: PathBuf::from("files"),
            library_root,
        }
    }

    #[test]
    fn absolute_path_builds_library_reference() {
        let config = config(PathBuf::from("/storage"), PathBuf::from("/srv"));
        let (file_ref, candidate, _) = build_reference("/srv/anime/foo/bar.pdf", &config).unwrap();

        assert_eq!(file_ref.root, FileRefRoot::Library);
        assert_eq!(file_ref.relative_path, "anime/foo/bar.pdf");
        assert_eq!(candidate, PathBuf::from("/srv/anime/foo/bar.pdf"));
    }

    #[test]
    fn relative_path_builds_storage_reference() {
        let config = config(PathBuf::from("/storage"), PathBuf::from("/srv"));
        let source = "b6b6f9a0/3c2b1a09.pdf";
        let (file_ref, candidate, _) = build_reference(source, &config).unwrap();

        assert_eq!(file_ref.root, FileRefRoot::Storage);
        assert_eq!(file_ref.relative_path, "files/b6b6f9a0/3c2b1a09.pdf");
        assert_eq!(candidate, PathBuf::from("/storage/files").join(source));
    }

    #[test]
    fn parent_directory_components_are_rejected() {
        let config = config(PathBuf::from("/storage"), PathBuf::from("/srv"));

        for path in ["../../etc/passwd", "files/../../../etc/passwd"] {
            let error = build_reference(path, &config).unwrap_err();
            assert_eq!(error.error.code, "VALIDATION_ERROR");
        }
    }

    #[test]
    fn absolute_path_outside_library_root_is_rejected() {
        let config = config(PathBuf::from("/storage"), PathBuf::from("/srv"));
        let error = build_reference("/etc/passwd", &config).unwrap_err();

        assert_eq!(error.error.code, "VALIDATION_ERROR");
    }

    #[test]
    fn unsafe_storage_subdirectory_is_rejected() {
        let mut config = config(PathBuf::from("/storage"), PathBuf::from("/srv"));
        config.storage_subdir_files = PathBuf::from("../escape");

        let error = build_reference("item/file.pdf", &config).unwrap_err();
        assert_eq!(error.error.code, "VALIDATION_ERROR");
    }

    #[test]
    fn missing_file_returns_unprocessable_entity() {
        let storage = tempfile::tempdir().unwrap();
        let library = tempfile::tempdir().unwrap();
        let config = config(storage.path().to_path_buf(), library.path().to_path_buf());

        let error = resolve("missing/file.pdf", &config).unwrap_err();
        assert_eq!(error.error.code, "UNPROCESSABLE_ENTITY");
    }

    #[test]
    fn directory_returns_unprocessable_entity() {
        let storage = tempfile::tempdir().unwrap();
        let library = tempfile::tempdir().unwrap();
        let directory = storage.path().join("files/item/directory.pdf");
        fs::create_dir_all(&directory).unwrap();
        let config = config(storage.path().to_path_buf(), library.path().to_path_buf());

        let error = resolve("item/directory.pdf", &config).unwrap_err();
        assert_eq!(error.error.code, "UNPROCESSABLE_ENTITY");
    }

    #[test]
    fn resolves_storage_and_library_files_with_sizes() {
        let storage = tempfile::tempdir().unwrap();
        let library = tempfile::tempdir().unwrap();
        let storage_file = storage.path().join("files/item/upload.pdf");
        let library_file = library.path().join("anime/linked.pdf");
        fs::create_dir_all(storage_file.parent().unwrap()).unwrap();
        fs::create_dir_all(library_file.parent().unwrap()).unwrap();
        fs::write(&storage_file, b"storage").unwrap();
        fs::write(&library_file, b"library-file").unwrap();
        let config = config(storage.path().to_path_buf(), library.path().to_path_buf());

        let storage_result = resolve("item/upload.pdf", &config).unwrap();
        let library_result = resolve(library_file.to_str().unwrap(), &config).unwrap();

        assert_eq!(storage_result.file_ref.root, FileRefRoot::Storage);
        assert_eq!(
            storage_result.file_ref.relative_path,
            "files/item/upload.pdf"
        );
        assert_eq!(storage_result.size_bytes, 7);
        assert_eq!(
            storage_result.absolute_path,
            fs::canonicalize(storage_file).unwrap()
        );
        assert_eq!(library_result.file_ref.root, FileRefRoot::Library);
        assert_eq!(library_result.file_ref.relative_path, "anime/linked.pdf");
        assert_eq!(library_result.size_bytes, 12);
        assert_eq!(
            library_result.absolute_path,
            fs::canonicalize(library_file).unwrap()
        );
    }
}
