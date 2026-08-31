//! Persistent guest storage for M32.
//!
//! This crate owns backend-independent persistence for guest RMS/database state and
//! guest filesystem bytes. WIE-specific adapters stay in `m32-wie-adapter`.

use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use m32_emulator_api::{
    GuestDatabaseError, GuestDatabaseHost, GuestDatabaseRecordId, GuestDatabaseRepositoryHost, GuestFilesystemError,
    GuestFilesystemHost, HostFuture,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

pub const STORAGE_DATABASE_FILE_NAME: &str = "storage.sqlite3";
pub const GUEST_FILES_DIRECTORY_NAME: &str = "guest-files";
pub const STORAGE_SCHEMA_VERSION: i64 = 1;
pub const SQLITE_BUSY_TIMEOUT_MS: u64 = 2_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoragePaths {
    pub root: PathBuf,
    pub database_file: PathBuf,
    pub guest_files_dir: PathBuf,
}

impl StoragePaths {
    #[must_use]
    pub fn from_m32_root(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            database_file: root.join(STORAGE_DATABASE_FILE_NAME),
            guest_files_dir: root.join(GUEST_FILES_DIRECTORY_NAME),
        }
    }

    fn ensure_directories(&self) -> Result<(), StorageInitError> {
        fs::create_dir_all(&self.root).map_err(|error| {
            StorageInitError::new(format!(
                "failed to create storage root '{}': {error}",
                self.root.display()
            ))
        })?;

        fs::create_dir_all(&self.guest_files_dir).map_err(|error| {
            StorageInitError::new(format!(
                "failed to create guest-files directory '{}': {error}",
                self.guest_files_dir.display()
            ))
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlitePolicySnapshot {
    pub journal_mode: String,
    pub foreign_keys: bool,
    pub busy_timeout_ms: u64,
    pub schema_version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageInitError {
    message: String,
}

impl StorageInitError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for StorageInitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for StorageInitError {}

#[derive(Debug, Clone)]
pub struct PersistentGuestStorage {
    paths: StoragePaths,
    database_repository: SqliteGuestDatabaseRepository,
    filesystem: DiskGuestFilesystem,
}

impl PersistentGuestStorage {
    pub fn open(m32_root: &Path) -> Result<Self, StorageInitError> {
        let paths = StoragePaths::from_m32_root(m32_root);
        paths.ensure_directories()?;

        initialize_database(&paths.database_file)?;

        Ok(Self {
            database_repository: SqliteGuestDatabaseRepository {
                database_file: paths.database_file.clone(),
            },
            filesystem: DiskGuestFilesystem {
                guest_files_dir: paths.guest_files_dir.clone(),
            },
            paths,
        })
    }

    #[must_use]
    pub fn paths(&self) -> &StoragePaths {
        &self.paths
    }

    #[must_use]
    pub fn database_repository(&self) -> SqliteGuestDatabaseRepository {
        self.database_repository.clone()
    }

    #[must_use]
    pub fn filesystem(&self) -> DiskGuestFilesystem {
        self.filesystem.clone()
    }

    pub fn sqlite_policy(&self) -> Result<SqlitePolicySnapshot, StorageInitError> {
        inspect_sqlite_policy(&self.paths.database_file)
    }
}

#[derive(Debug, Clone)]
pub struct SqliteGuestDatabaseRepository {
    database_file: PathBuf,
}

impl SqliteGuestDatabaseRepository {
    pub fn open_path(database_file: &Path) -> Result<Self, StorageInitError> {
        if let Some(parent) = database_file.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                StorageInitError::new(format!(
                    "failed to create database parent '{}': {error}",
                    parent.display()
                ))
            })?;
        }

        initialize_database(database_file)?;

        Ok(Self {
            database_file: database_file.to_path_buf(),
        })
    }
}

#[derive(Debug, Clone)]
struct SqliteGuestDatabase {
    database_file: PathBuf,
    app_id: String,
    name: String,
}

#[derive(Debug, Clone)]
pub struct DiskGuestFilesystem {
    guest_files_dir: PathBuf,
}

impl DiskGuestFilesystem {
    pub fn open_root(guest_files_dir: &Path) -> Result<Self, StorageInitError> {
        fs::create_dir_all(guest_files_dir).map_err(|error| {
            StorageInitError::new(format!(
                "failed to create guest filesystem root '{}': {error}",
                guest_files_dir.display()
            ))
        })?;

        Ok(Self {
            guest_files_dir: guest_files_dir.to_path_buf(),
        })
    }

    fn resolve_file_path(&self, aid: &str, guest_path: &str) -> Result<PathBuf, GuestFilesystemError> {
        if aid.is_empty() {
            return Err(filesystem_error("AID must not be empty"));
        }

        if aid.contains('\0') {
            return Err(filesystem_error("AID contains NUL"));
        }

        if guest_path.contains('\0') {
            return Err(filesystem_error("guest path contains NUL"));
        }

        let mut resolved = self
            .guest_files_dir
            .join(format!("a-{}", encode_component(aid.as_bytes())));

        let mut component_count = 0_usize;

        for component in guest_path.split(['/', '\\']) {
            if component.is_empty() {
                continue;
            }

            if matches!(component, "." | "..") {
                return Err(filesystem_error("guest path contains a traversal component"));
            }

            resolved.push(format!("c-{}", encode_component(component.as_bytes())));
            component_count += 1;
        }

        if component_count == 0 {
            return Err(filesystem_error("guest path must identify a file"));
        }

        Ok(resolved)
    }
}

impl GuestDatabaseRepositoryHost for SqliteGuestDatabaseRepository {
    fn open<'a>(
        &'a self,
        name: &'a str,
        app_id: &'a str,
    ) -> HostFuture<'a, Result<Box<dyn GuestDatabaseHost>, GuestDatabaseError>> {
        Box::pin(async move {
            validate_database_identity(name, app_id)?;

            let connection = open_database_connection_for_guest(&self.database_file)?;

            connection
                .execute(
                    "INSERT OR IGNORE INTO guest_databases(app_id, name, next_id)
                     VALUES (?1, ?2, 1)",
                    params![app_id, name],
                )
                .map_err(|error| database_error("failed to open guest database", error))?;

            Ok(Box::new(SqliteGuestDatabase {
                database_file: self.database_file.clone(),
                app_id: app_id.to_owned(),
                name: name.to_owned(),
            }) as Box<dyn GuestDatabaseHost>)
        })
    }

    fn exists<'a>(&'a self, name: &'a str, app_id: &'a str) -> HostFuture<'a, Result<bool, GuestDatabaseError>> {
        Box::pin(async move {
            validate_database_identity(name, app_id)?;

            let connection = open_database_connection_for_guest(&self.database_file)?;

            let exists = connection
                .query_row(
                    "SELECT 1 FROM guest_databases
                     WHERE app_id = ?1 AND name = ?2
                     LIMIT 1",
                    params![app_id, name],
                    |_| Ok(()),
                )
                .optional()
                .map_err(|error| database_error("failed to test guest database existence", error))?
                .is_some();

            Ok(exists)
        })
    }

    fn delete<'a>(&'a self, name: &'a str, app_id: &'a str) -> HostFuture<'a, Result<bool, GuestDatabaseError>> {
        Box::pin(async move {
            validate_database_identity(name, app_id)?;

            let connection = open_database_connection_for_guest(&self.database_file)?;

            let deleted = connection
                .execute(
                    "DELETE FROM guest_databases
                     WHERE app_id = ?1 AND name = ?2",
                    params![app_id, name],
                )
                .map_err(|error| database_error("failed to delete guest database", error))?;

            Ok(deleted == 1)
        })
    }

    fn usage<'a>(&'a self, app_id: &'a str) -> HostFuture<'a, Result<u64, GuestDatabaseError>> {
        Box::pin(async move {
            validate_non_empty_no_nul("app_id", app_id)?;

            let connection = open_database_connection_for_guest(&self.database_file)?;

            let usage: i64 = connection
                .query_row(
                    "SELECT COALESCE(SUM(length(data)), 0)
                     FROM guest_records
                     WHERE app_id = ?1",
                    params![app_id],
                    |row| row.get(0),
                )
                .map_err(|error| database_error("failed to query guest database usage", error))?;

            u64::try_from(usage).map_err(|error| database_error("guest database usage overflow", error))
        })
    }
}

impl GuestDatabaseHost for SqliteGuestDatabase {
    fn next_id<'a>(&'a self) -> HostFuture<'a, Result<GuestDatabaseRecordId, GuestDatabaseError>> {
        Box::pin(async move {
            let connection = open_database_connection_for_guest(&self.database_file)?;

            let raw_id: i64 = connection
                .query_row(
                    "SELECT next_id FROM guest_databases
                     WHERE app_id = ?1 AND name = ?2",
                    params![&self.app_id, &self.name],
                    |row| row.get(0),
                )
                .map_err(|error| database_error("failed to query next guest record id", error))?;

            checked_record_id(raw_id)
        })
    }

    fn add<'a>(&'a mut self, data: &'a [u8]) -> HostFuture<'a, Result<GuestDatabaseRecordId, GuestDatabaseError>> {
        Box::pin(async move {
            let mut connection = open_database_connection_for_guest(&self.database_file)?;

            let transaction = connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(|error| database_error("failed to begin guest record transaction", error))?;

            let raw_id: i64 = transaction
                .query_row(
                    "SELECT next_id FROM guest_databases
                     WHERE app_id = ?1 AND name = ?2",
                    params![&self.app_id, &self.name],
                    |row| row.get(0),
                )
                .map_err(|error| database_error("failed to reserve guest record id", error))?;

            let id = checked_record_id(raw_id)?;

            transaction
                .execute(
                    "INSERT INTO guest_records(app_id, db_name, record_id, data)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![&self.app_id, &self.name, i64::from(id), data],
                )
                .map_err(|error| database_error("failed to insert guest record", error))?;

            transaction
                .execute(
                    "UPDATE guest_databases
                     SET next_id = next_id + 1
                     WHERE app_id = ?1 AND name = ?2",
                    params![&self.app_id, &self.name],
                )
                .map_err(|error| database_error("failed to advance guest record id", error))?;

            transaction
                .commit()
                .map_err(|error| database_error("failed to commit guest record", error))?;

            Ok(id)
        })
    }

    fn get<'a>(&'a self, id: GuestDatabaseRecordId) -> HostFuture<'a, Result<Option<Vec<u8>>, GuestDatabaseError>> {
        Box::pin(async move {
            let connection = open_database_connection_for_guest(&self.database_file)?;

            connection
                .query_row(
                    "SELECT data FROM guest_records
                     WHERE app_id = ?1 AND db_name = ?2 AND record_id = ?3",
                    params![&self.app_id, &self.name, i64::from(id)],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|error| database_error("failed to read guest record", error))
        })
    }

    fn set<'a>(
        &'a mut self,
        id: GuestDatabaseRecordId,
        data: &'a [u8],
    ) -> HostFuture<'a, Result<bool, GuestDatabaseError>> {
        Box::pin(async move {
            let connection = open_database_connection_for_guest(&self.database_file)?;

            let updated = connection
                .execute(
                    "UPDATE guest_records
                     SET data = ?4
                     WHERE app_id = ?1 AND db_name = ?2 AND record_id = ?3",
                    params![&self.app_id, &self.name, i64::from(id), data],
                )
                .map_err(|error| database_error("failed to update guest record", error))?;

            Ok(updated == 1)
        })
    }

    fn delete<'a>(&'a mut self, id: GuestDatabaseRecordId) -> HostFuture<'a, Result<bool, GuestDatabaseError>> {
        Box::pin(async move {
            let connection = open_database_connection_for_guest(&self.database_file)?;

            let deleted = connection
                .execute(
                    "DELETE FROM guest_records
                     WHERE app_id = ?1 AND db_name = ?2 AND record_id = ?3",
                    params![&self.app_id, &self.name, i64::from(id)],
                )
                .map_err(|error| database_error("failed to delete guest record", error))?;

            Ok(deleted == 1)
        })
    }

    fn record_ids<'a>(&'a self) -> HostFuture<'a, Result<Vec<GuestDatabaseRecordId>, GuestDatabaseError>> {
        Box::pin(async move {
            let connection = open_database_connection_for_guest(&self.database_file)?;

            let mut statement = connection
                .prepare(
                    "SELECT record_id FROM guest_records
                     WHERE app_id = ?1 AND db_name = ?2
                     ORDER BY record_id ASC",
                )
                .map_err(|error| database_error("failed to prepare guest record id query", error))?;

            let rows = statement
                .query_map(params![&self.app_id, &self.name], |row| row.get::<_, i64>(0))
                .map_err(|error| database_error("failed to query guest record ids", error))?;

            let mut ids = Vec::new();

            for row in rows {
                let raw_id = row.map_err(|error| database_error("failed to decode guest record id", error))?;
                ids.push(checked_record_id(raw_id)?);
            }

            Ok(ids)
        })
    }
}

impl GuestFilesystemHost for DiskGuestFilesystem {
    fn exists<'a>(&'a self, aid: &'a str, path: &'a str) -> HostFuture<'a, Result<bool, GuestFilesystemError>> {
        Box::pin(async move {
            let resolved = self.resolve_file_path(aid, path)?;

            match fs::metadata(&resolved) {
                Ok(metadata) => Ok(metadata.is_file()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
                Err(error) => Err(filesystem_io_error("failed to inspect guest file", &resolved, error)),
            }
        })
    }

    fn size<'a>(&'a self, aid: &'a str, path: &'a str) -> HostFuture<'a, Result<Option<usize>, GuestFilesystemError>> {
        Box::pin(async move {
            let resolved = self.resolve_file_path(aid, path)?;

            match fs::metadata(&resolved) {
                Ok(metadata) if metadata.is_file() => usize::try_from(metadata.len()).map(Some).map_err(|error| {
                    filesystem_error(format!("guest file size overflow '{}': {error}", resolved.display()))
                }),
                Ok(_) => Ok(None),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(filesystem_io_error("failed to query guest file size", &resolved, error)),
            }
        })
    }

    fn read<'a>(
        &'a self,
        aid: &'a str,
        path: &'a str,
        offset: usize,
        count: usize,
        buf: &'a mut [u8],
    ) -> HostFuture<'a, Result<Option<usize>, GuestFilesystemError>> {
        Box::pin(async move {
            if buf.len() < count {
                return Err(filesystem_error(format!(
                    "guest read buffer too small: buffer={}, requested={count}",
                    buf.len()
                )));
            }

            let resolved = self.resolve_file_path(aid, path)?;

            let mut file = match File::open(&resolved) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(None);
                }
                Err(error) => {
                    return Err(filesystem_io_error(
                        "failed to open guest file for read",
                        &resolved,
                        error,
                    ));
                }
            };

            let size = file
                .metadata()
                .map_err(|error| filesystem_io_error("failed to inspect guest file before read", &resolved, error))?
                .len();

            let offset_u64 = u64::try_from(offset)
                .map_err(|error| filesystem_error(format!("guest read offset overflow: {error}")))?;

            if offset_u64 >= size {
                return Ok(Some(0));
            }

            file.seek(SeekFrom::Start(offset_u64))
                .map_err(|error| filesystem_io_error("failed to seek guest file for read", &resolved, error))?;

            let read = file
                .read(&mut buf[..count])
                .map_err(|error| filesystem_io_error("failed to read guest file", &resolved, error))?;

            Ok(Some(read))
        })
    }

    fn write<'a>(
        &'a self,
        aid: &'a str,
        path: &'a str,
        offset: usize,
        data: &'a [u8],
    ) -> HostFuture<'a, Result<usize, GuestFilesystemError>> {
        Box::pin(async move {
            let resolved = self.resolve_file_path(aid, path)?;

            if let Some(parent) = resolved.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    filesystem_io_error("failed to create guest file parent directory", parent, error)
                })?;
            }

            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&resolved)
                .map_err(|error| filesystem_io_error("failed to open guest file for write", &resolved, error))?;

            let offset_u64 = u64::try_from(offset)
                .map_err(|error| filesystem_error(format!("guest write offset overflow: {error}")))?;

            let current_len = file
                .metadata()
                .map_err(|error| filesystem_io_error("failed to inspect guest file before write", &resolved, error))?
                .len();

            if offset_u64 > current_len {
                file.set_len(offset_u64).map_err(|error| {
                    filesystem_io_error("failed to zero-extend guest file before write", &resolved, error)
                })?;
            }

            file.seek(SeekFrom::Start(offset_u64))
                .map_err(|error| filesystem_io_error("failed to seek guest file for write", &resolved, error))?;

            file.write_all(data)
                .map_err(|error| filesystem_io_error("failed to write guest file", &resolved, error))?;

            file.flush()
                .map_err(|error| filesystem_io_error("failed to flush guest file", &resolved, error))?;

            file.sync_data()
                .map_err(|error| filesystem_io_error("failed to sync guest file", &resolved, error))?;

            Ok(data.len())
        })
    }

    fn truncate<'a>(
        &'a self,
        aid: &'a str,
        path: &'a str,
        len: usize,
    ) -> HostFuture<'a, Result<(), GuestFilesystemError>> {
        Box::pin(async move {
            let resolved = self.resolve_file_path(aid, path)?;

            if let Some(parent) = resolved.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    filesystem_io_error("failed to create guest file parent directory", parent, error)
                })?;
            }

            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false)
                .open(&resolved)
                .map_err(|error| filesystem_io_error("failed to open guest file for truncate", &resolved, error))?;

            let len_u64 = u64::try_from(len)
                .map_err(|error| filesystem_error(format!("guest truncate length overflow: {error}")))?;

            file.set_len(len_u64)
                .map_err(|error| filesystem_io_error("failed to resize guest file", &resolved, error))?;

            file.sync_data()
                .map_err(|error| filesystem_io_error("failed to sync resized guest file", &resolved, error))
        })
    }
}

fn initialize_database(database_file: &Path) -> Result<(), StorageInitError> {
    let mut connection = open_storage_connection(database_file)?;

    let journal_mode: String = connection
        .query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))
        .map_err(|error| StorageInitError::new(format!("failed to enable SQLite WAL mode: {error}")))?;

    if !journal_mode.eq_ignore_ascii_case("wal") {
        return Err(StorageInitError::new(format!(
            "SQLite refused WAL mode; active mode is '{journal_mode}'"
        )));
    }

    let schema_version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| StorageInitError::new(format!("failed to query SQLite schema version: {error}")))?;

    match schema_version {
        0 => migrate_schema_v1(&mut connection)?,
        STORAGE_SCHEMA_VERSION => {}
        other => {
            return Err(StorageInitError::new(format!(
                "unsupported M32 storage schema version {other}"
            )));
        }
    }

    Ok(())
}

fn migrate_schema_v1(connection: &mut Connection) -> Result<(), StorageInitError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| StorageInitError::new(format!("failed to begin storage schema migration: {error}")))?;

    transaction
        .execute_batch(
            "
            CREATE TABLE guest_databases (
                app_id  TEXT NOT NULL,
                name    TEXT NOT NULL,
                next_id INTEGER NOT NULL DEFAULT 1
                        CHECK(next_id >= 1 AND next_id <= 4294967296),
                PRIMARY KEY(app_id, name)
            ) WITHOUT ROWID;

            CREATE TABLE guest_records (
                app_id    TEXT NOT NULL,
                db_name   TEXT NOT NULL,
                record_id INTEGER NOT NULL
                          CHECK(record_id >= 1 AND record_id <= 4294967295),
                data      BLOB NOT NULL,
                PRIMARY KEY(app_id, db_name, record_id),
                FOREIGN KEY(app_id, db_name)
                    REFERENCES guest_databases(app_id, name)
                    ON DELETE CASCADE
            ) WITHOUT ROWID;

            PRAGMA user_version = 1;
            ",
        )
        .map_err(|error| StorageInitError::new(format!("failed to create storage schema v1: {error}")))?;

    transaction
        .commit()
        .map_err(|error| StorageInitError::new(format!("failed to commit storage schema v1: {error}")))
}

fn open_storage_connection(database_file: &Path) -> Result<Connection, StorageInitError> {
    let connection = Connection::open(database_file).map_err(|error| {
        StorageInitError::new(format!(
            "failed to open SQLite database '{}': {error}",
            database_file.display()
        ))
    })?;

    configure_connection(&connection).map_err(StorageInitError::new)?;

    Ok(connection)
}

fn open_database_connection_for_guest(database_file: &Path) -> Result<Connection, GuestDatabaseError> {
    let connection =
        Connection::open(database_file).map_err(|error| database_error("failed to open M32 SQLite storage", error))?;

    configure_connection(&connection).map_err(GuestDatabaseError::operation_failed)?;

    Ok(connection)
}

fn configure_connection(connection: &Connection) -> Result<(), String> {
    connection
        .busy_timeout(Duration::from_millis(SQLITE_BUSY_TIMEOUT_MS))
        .map_err(|error| format!("failed to set SQLite busy timeout: {error}"))?;

    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(|error| format!("failed to enable SQLite foreign keys: {error}"))?;

    Ok(())
}

fn inspect_sqlite_policy(database_file: &Path) -> Result<SqlitePolicySnapshot, StorageInitError> {
    let connection = open_storage_connection(database_file)?;

    let journal_mode: String = connection
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .map_err(|error| StorageInitError::new(format!("failed to inspect SQLite journal mode: {error}")))?;

    let foreign_keys: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .map_err(|error| StorageInitError::new(format!("failed to inspect SQLite foreign-key mode: {error}")))?;

    let busy_timeout_ms: i64 = connection
        .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
        .map_err(|error| StorageInitError::new(format!("failed to inspect SQLite busy timeout: {error}")))?;

    let schema_version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| StorageInitError::new(format!("failed to inspect SQLite schema version: {error}")))?;

    Ok(SqlitePolicySnapshot {
        journal_mode,
        foreign_keys: foreign_keys == 1,
        busy_timeout_ms: u64::try_from(busy_timeout_ms)
            .map_err(|error| StorageInitError::new(format!("SQLite busy timeout cannot be represented: {error}")))?,
        schema_version,
    })
}

fn validate_database_identity(name: &str, app_id: &str) -> Result<(), GuestDatabaseError> {
    validate_non_empty_no_nul("database name", name)?;
    validate_non_empty_no_nul("app_id", app_id)
}

fn validate_non_empty_no_nul(field: &str, value: &str) -> Result<(), GuestDatabaseError> {
    if value.is_empty() {
        return Err(GuestDatabaseError::operation_failed(format!(
            "{field} must not be empty"
        )));
    }

    if value.contains('\0') {
        return Err(GuestDatabaseError::operation_failed(format!(
            "{field} must not contain NUL"
        )));
    }

    Ok(())
}

fn checked_record_id(raw_id: i64) -> Result<GuestDatabaseRecordId, GuestDatabaseError> {
    if raw_id <= 0 {
        return Err(GuestDatabaseError::operation_failed(format!(
            "invalid non-positive guest record id {raw_id}"
        )));
    }

    GuestDatabaseRecordId::try_from(raw_id).map_err(|error| database_error("guest record id overflow", error))
}

fn database_error(context: &str, error: impl fmt::Display) -> GuestDatabaseError {
    GuestDatabaseError::operation_failed(format!("{context}: {error}"))
}

fn filesystem_error(message: impl Into<String>) -> GuestFilesystemError {
    GuestFilesystemError::operation_failed(message)
}

fn filesystem_io_error(context: &str, path: &Path, error: std::io::Error) -> GuestFilesystemError {
    filesystem_error(format!("{context} '{}': {error}", path.display()))
}

fn encode_component(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "00".to_owned();
    }

    let mut encoded = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        use std::fmt::Write as _;
        write!(encoded, "{byte:02X}").expect("writing hexadecimal bytes into String cannot fail");
    }

    encoded
}

#[cfg(test)]
mod tests {
    use std::{
        future::Future,
        pin::Pin,
        sync::atomic::{AtomicU64, Ordering},
        task::{Context, Poll, Waker},
    };

    use super::*;

    static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(1);

    struct TempRoot {
        path: PathBuf,
    }

    impl TempRoot {
        fn new() -> Self {
            let id = NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("m32-storage-test-{}-{id}", std::process::id()));

            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("temporary storage root must be created");

            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn block_on_ready<T>(mut future: Pin<Box<dyn Future<Output = T> + Send + '_>>) -> T {
        let mut context = Context::from_waker(Waker::noop());

        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("M32 storage host future unexpectedly pending"),
        }
    }

    fn open_database(
        repository: &SqliteGuestDatabaseRepository,
        name: &str,
        app_id: &str,
    ) -> Box<dyn GuestDatabaseHost> {
        block_on_ready(repository.open(name, app_id)).expect("guest database must open")
    }

    #[test]
    fn storage_paths_are_rooted_under_m32_root() {
        let root = Path::new(r"C:\Users\M32Test\AppData\Local\M32");
        let paths = StoragePaths::from_m32_root(root);

        assert_eq!(paths.root, root);
        assert_eq!(paths.database_file, root.join(STORAGE_DATABASE_FILE_NAME));
        assert_eq!(paths.guest_files_dir, root.join(GUEST_FILES_DIRECTORY_NAME));
        assert_eq!(STORAGE_DATABASE_FILE_NAME, "storage.sqlite3");
        assert_eq!(GUEST_FILES_DIRECTORY_NAME, "guest-files");
    }

    #[test]
    fn sqlite_policy_is_wal_foreign_keys_busy_timeout_2000_and_schema_v1() {
        let temp = TempRoot::new();
        let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
        let policy = storage.sqlite_policy().expect("policy must inspect");

        assert_eq!(policy.journal_mode.to_ascii_lowercase(), "wal");
        assert!(policy.foreign_keys);
        assert_eq!(policy.busy_timeout_ms, SQLITE_BUSY_TIMEOUT_MS);
        assert_eq!(policy.schema_version, STORAGE_SCHEMA_VERSION);
        assert_eq!(SQLITE_BUSY_TIMEOUT_MS, 2_000);
        assert_eq!(STORAGE_SCHEMA_VERSION, 1);
    }

    #[test]
    fn repository_open_exists_delete_are_scoped_by_app_and_name() {
        let temp = TempRoot::new();
        let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
        let repository = storage.database_repository();

        assert!(!block_on_ready(repository.exists("save", "game-a")).unwrap());

        let _ = open_database(&repository, "save", "game-a");

        assert!(block_on_ready(repository.exists("save", "game-a")).unwrap());
        assert!(!block_on_ready(repository.exists("save", "game-b")).unwrap());
        assert!(!block_on_ready(repository.exists("other", "game-a")).unwrap());

        assert!(block_on_ready(repository.delete("save", "game-a")).unwrap());
        assert!(!block_on_ready(repository.exists("save", "game-a")).unwrap());
        assert!(!block_on_ready(repository.delete("save", "game-a")).unwrap());
    }

    #[test]
    fn record_ids_start_at_one_and_do_not_reuse_deleted_ids() {
        let temp = TempRoot::new();
        let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
        let repository = storage.database_repository();
        let mut database = open_database(&repository, "save", "game-a");

        assert_eq!(block_on_ready(database.next_id()).unwrap(), 1);

        let first = block_on_ready(database.add(b"one")).unwrap();
        let second = block_on_ready(database.add(b"two")).unwrap();

        assert_eq!(first, 1);
        assert_eq!(second, 2);
        assert_eq!(block_on_ready(database.next_id()).unwrap(), 3);

        assert!(block_on_ready(database.delete(first)).unwrap());

        let third = block_on_ready(database.add(b"three")).unwrap();
        assert_eq!(third, 3);
        assert_eq!(block_on_ready(database.next_id()).unwrap(), 4);
    }

    #[test]
    fn record_get_set_delete_and_sorted_ids_round_trip() {
        let temp = TempRoot::new();
        let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
        let repository = storage.database_repository();
        let mut database = open_database(&repository, "save", "game-a");

        let first = block_on_ready(database.add(b"alpha")).unwrap();
        let second = block_on_ready(database.add(b"beta")).unwrap();
        let third = block_on_ready(database.add(b"gamma")).unwrap();

        assert_eq!(
            block_on_ready(database.record_ids()).unwrap(),
            vec![first, second, third]
        );
        assert_eq!(block_on_ready(database.get(second)).unwrap(), Some(b"beta".to_vec()));

        assert!(block_on_ready(database.set(second, b"BETA")).unwrap());
        assert_eq!(block_on_ready(database.get(second)).unwrap(), Some(b"BETA".to_vec()));
        assert!(!block_on_ready(database.set(999, b"missing")).unwrap());

        assert!(block_on_ready(database.delete(first)).unwrap());
        assert!(!block_on_ready(database.delete(first)).unwrap());

        assert_eq!(block_on_ready(database.record_ids()).unwrap(), vec![second, third]);
    }

    #[test]
    fn usage_sums_record_payload_bytes_per_app() {
        let temp = TempRoot::new();
        let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
        let repository = storage.database_repository();

        let mut first = open_database(&repository, "save-a", "game-a");
        let mut second = open_database(&repository, "save-b", "game-a");
        let mut other = open_database(&repository, "save", "game-b");

        block_on_ready(first.add(&[1, 2, 3])).unwrap();
        block_on_ready(second.add(&[4, 5, 6, 7, 8])).unwrap();
        block_on_ready(other.add(&[9; 32])).unwrap();

        assert_eq!(block_on_ready(repository.usage("game-a")).unwrap(), 8);
        assert_eq!(block_on_ready(repository.usage("game-b")).unwrap(), 32);
    }

    #[test]
    fn database_persists_across_reopen() {
        let temp = TempRoot::new();

        {
            let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
            let repository = storage.database_repository();
            let mut database = open_database(&repository, "save", "game-a");

            let id = block_on_ready(database.add(b"persistent-save")).unwrap();
            assert_eq!(id, 1);
        }

        let reopened = PersistentGuestStorage::open(temp.path()).expect("storage must reopen");
        let repository = reopened.database_repository();
        assert!(block_on_ready(repository.exists("save", "game-a")).unwrap());

        let database = open_database(&repository, "save", "game-a");
        assert_eq!(
            block_on_ready(database.get(1)).unwrap(),
            Some(b"persistent-save".to_vec())
        );
        assert_eq!(block_on_ready(database.next_id()).unwrap(), 2);
    }

    #[test]
    fn filesystem_write_read_size_and_exists_round_trip() {
        let temp = TempRoot::new();
        let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
        let filesystem = storage.filesystem();

        assert!(!block_on_ready(filesystem.exists("game-a", "/save/data.bin")).unwrap());

        assert_eq!(
            block_on_ready(filesystem.write("game-a", "/save/data.bin", 0, b"abcdef")).unwrap(),
            6
        );

        assert!(block_on_ready(filesystem.exists("game-a", "/save/data.bin")).unwrap());
        assert_eq!(
            block_on_ready(filesystem.size("game-a", "/save/data.bin")).unwrap(),
            Some(6)
        );

        let mut buffer = [0_u8; 4];
        assert_eq!(
            block_on_ready(filesystem.read("game-a", "/save/data.bin", 1, 4, &mut buffer)).unwrap(),
            Some(4)
        );
        assert_eq!(&buffer, b"bcde");
    }

    #[test]
    fn filesystem_write_past_eof_zero_fills_gap() {
        let temp = TempRoot::new();
        let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
        let filesystem = storage.filesystem();

        block_on_ready(filesystem.write("game-a", "gap.bin", 4, b"XY")).unwrap();

        assert_eq!(block_on_ready(filesystem.size("game-a", "gap.bin")).unwrap(), Some(6));

        let mut buffer = [0xFF_u8; 6];
        assert_eq!(
            block_on_ready(filesystem.read("game-a", "gap.bin", 0, 6, &mut buffer)).unwrap(),
            Some(6)
        );
        assert_eq!(&buffer, &[0, 0, 0, 0, b'X', b'Y']);
    }

    #[test]
    fn filesystem_truncate_shrinks_extends_and_creates() {
        let temp = TempRoot::new();
        let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
        let filesystem = storage.filesystem();

        block_on_ready(filesystem.write("game-a", "resize.bin", 0, b"abcdef")).unwrap();

        block_on_ready(filesystem.truncate("game-a", "resize.bin", 3)).unwrap();
        assert_eq!(
            block_on_ready(filesystem.size("game-a", "resize.bin")).unwrap(),
            Some(3)
        );

        block_on_ready(filesystem.truncate("game-a", "resize.bin", 7)).unwrap();

        let mut buffer = [0xFF_u8; 7];
        block_on_ready(filesystem.read("game-a", "resize.bin", 0, 7, &mut buffer)).unwrap();

        assert_eq!(&buffer, &[b'a', b'b', b'c', 0, 0, 0, 0]);

        block_on_ready(filesystem.truncate("game-a", "new-empty.bin", 0)).unwrap();
        assert_eq!(
            block_on_ready(filesystem.size("game-a", "new-empty.bin")).unwrap(),
            Some(0)
        );
    }

    #[test]
    fn filesystem_missing_and_eof_read_semantics_are_exact() {
        let temp = TempRoot::new();
        let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
        let filesystem = storage.filesystem();

        let mut buffer = [0_u8; 8];

        assert_eq!(
            block_on_ready(filesystem.read("game-a", "missing.bin", 0, 8, &mut buffer)).unwrap(),
            None
        );

        block_on_ready(filesystem.write("game-a", "data.bin", 0, b"abc")).unwrap();

        assert_eq!(
            block_on_ready(filesystem.read("game-a", "data.bin", 3, 8, &mut buffer)).unwrap(),
            Some(0)
        );

        assert_eq!(
            block_on_ready(filesystem.read("game-a", "data.bin", 100, 8, &mut buffer)).unwrap(),
            Some(0)
        );
    }

    #[test]
    fn filesystem_rejects_parent_traversal_and_empty_file_path() {
        let temp = TempRoot::new();
        let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
        let filesystem = storage.filesystem();

        assert!(block_on_ready(filesystem.write("game-a", "../escape.bin", 0, b"x")).is_err());
        assert!(block_on_ready(filesystem.write("game-a", "safe/../escape.bin", 0, b"x")).is_err());
        assert!(block_on_ready(filesystem.write("game-a", "/", 0, b"x")).is_err());
    }

    #[test]
    fn filesystem_aids_are_isolated_logically() {
        let temp = TempRoot::new();
        let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
        let filesystem = storage.filesystem();

        block_on_ready(filesystem.write("game-a", "save.bin", 0, b"AAA")).unwrap();
        block_on_ready(filesystem.write("game-b", "save.bin", 0, b"BBB")).unwrap();

        let mut a = [0_u8; 3];
        let mut b = [0_u8; 3];

        block_on_ready(filesystem.read("game-a", "save.bin", 0, 3, &mut a)).unwrap();
        block_on_ready(filesystem.read("game-b", "save.bin", 0, 3, &mut b)).unwrap();

        assert_eq!(&a, b"AAA");
        assert_eq!(&b, b"BBB");
    }

    #[test]
    fn combined_storage_reopen_preserves_database_and_file_without_cross_game_leak() {
        let temp = TempRoot::new();

        {
            let storage = PersistentGuestStorage::open(temp.path()).expect("storage must open");
            let repository = storage.database_repository();
            let filesystem = storage.filesystem();

            let mut database = open_database(&repository, "rms", "game-a");
            block_on_ready(database.add(b"RMS-A")).unwrap();

            block_on_ready(filesystem.write("game-a", "save/state.bin", 0, b"FILE-A")).unwrap();
        }

        let reopened = PersistentGuestStorage::open(temp.path()).expect("storage must reopen");
        let repository = reopened.database_repository();
        let filesystem = reopened.filesystem();

        let database = open_database(&repository, "rms", "game-a");
        assert_eq!(block_on_ready(database.get(1)).unwrap(), Some(b"RMS-A".to_vec()));

        assert!(!block_on_ready(repository.exists("rms", "game-b")).unwrap());
        assert!(!block_on_ready(filesystem.exists("game-b", "save/state.bin")).unwrap());

        let mut buffer = [0_u8; 6];
        assert_eq!(
            block_on_ready(filesystem.read("game-a", "save/state.bin", 0, 6, &mut buffer)).unwrap(),
            Some(6)
        );
        assert_eq!(&buffer, b"FILE-A");
    }
}
