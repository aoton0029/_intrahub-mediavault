//! Service層: 複数API呼び出しの合成・名前→ID解決・冪等性・部分失敗は後続タスクで実装する

pub mod add_access_link;
pub mod attach;
pub mod collection_overview;
pub mod context;
pub mod create_item;
pub mod cursor;
pub mod health;
pub mod import_external_item;
pub mod organize;
pub mod relate_items;
pub mod resolve;
pub mod search_external_catalog;
pub mod search_library;
pub mod update_consumption;
