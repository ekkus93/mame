use rusqlite::{Connection, OptionalExtension};

use crate::errors::{AppError, AppResult};

pub const CATALOG_SCHEMA_VERSION: i64 = 4;

pub(crate) fn migrate(connection: &mut Connection) -> AppResult<()> {
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;\nPRAGMA journal_mode = WAL;\nPRAGMA synchronous = NORMAL;",
        )
        .map_err(|error| database_error("CATALOG_DATABASE_PRAGMA_FAILED", error))?;

    let mut version = current_version(connection)?;
    if version > CATALOG_SCHEMA_VERSION {
        return Err(AppError::new(
            "CATALOG_SCHEMA_UNSUPPORTED",
            format!(
                "Catalog schema version {version} is newer than supported version {CATALOG_SCHEMA_VERSION}."
            ),
        )
        .with_details(serde_json::json!({
            "foundVersion": version,
            "supportedVersion": CATALOG_SCHEMA_VERSION
        })));
    }

    while version < CATALOG_SCHEMA_VERSION {
        match version {
            0 => migrate_v0_to_v1(connection)?,
            1 => migrate_v1_to_v2(connection)?,
            2 => migrate_v2_to_v3(connection)?,
            3 => migrate_v3_to_v4(connection)?,
            unsupported => {
                return Err(AppError::new(
                    "CATALOG_SCHEMA_MIGRATION_MISSING",
                    "No catalog schema migration is available for the current database version.",
                )
                .with_details(serde_json::json!({
                    "foundVersion": unsupported,
                    "targetVersion": CATALOG_SCHEMA_VERSION
                })))
            }
        }
        version = current_version(connection)?;
    }

    Ok(())
}

fn current_version(connection: &Connection) -> AppResult<i64> {
    let exists: i64 = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'app_schema_version')",
            [],
            |row| row.get(0),
        )
        .map_err(|error| database_error("CATALOG_SCHEMA_READ_FAILED", error))?;

    if exists == 0 {
        return Ok(0);
    }

    connection
        .query_row(
            "SELECT version FROM app_schema_version WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| database_error("CATALOG_SCHEMA_READ_FAILED", error))?
        .ok_or_else(|| {
            AppError::new(
                "CATALOG_SCHEMA_INVALID",
                "The catalog schema version table exists but has no singleton version record.",
            )
        })
}

fn migrate_v0_to_v1(connection: &mut Connection) -> AppResult<()> {
    let transaction = connection
        .transaction()
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;

    transaction
        .execute_batch(CREATE_SCHEMA_V1)
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;
    transaction
        .execute(
            "INSERT INTO app_schema_version(singleton, version) VALUES (1, ?1)",
            [1_i64],
        )
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;
    transaction
        .commit()
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;
    Ok(())
}

fn migrate_v1_to_v2(connection: &mut Connection) -> AppResult<()> {
    let transaction = connection
        .transaction()
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;

    transaction
        .execute_batch(CREATE_SCHEMA_V2)
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;
    transaction
        .execute(
            "UPDATE app_schema_version SET version = ?1 WHERE singleton = 1",
            [2_i64],
        )
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;
    transaction
        .commit()
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;
    Ok(())
}

fn migrate_v2_to_v3(connection: &mut Connection) -> AppResult<()> {
    let transaction = connection
        .transaction()
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;

    transaction
        .execute_batch(CREATE_SCHEMA_V3)
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;
    transaction
        .execute(
            "UPDATE app_schema_version SET version = ?1 WHERE singleton = 1",
            [3_i64],
        )
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;
    transaction
        .commit()
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;
    Ok(())
}

fn migrate_v3_to_v4(connection: &mut Connection) -> AppResult<()> {
    let transaction = connection
        .transaction()
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;

    transaction
        .execute_batch(CREATE_SCHEMA_V4)
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;
    transaction
        .execute(
            "UPDATE app_schema_version SET version = ?1 WHERE singleton = 1",
            [4_i64],
        )
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;
    transaction
        .commit()
        .map_err(|error| database_error("CATALOG_SCHEMA_MIGRATION_FAILED", error))?;
    Ok(())
}

const CREATE_SCHEMA_V1: &str = r#"
CREATE TABLE app_schema_version (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    version INTEGER NOT NULL
);

CREATE TABLE metadata_generation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_kind TEXT NOT NULL,
    trust TEXT NOT NULL,
    executable_path TEXT NOT NULL,
    mame_version TEXT NOT NULL,
    mame_build TEXT,
    raw_version_line TEXT NOT NULL,
    listxml_build TEXT,
    mame_config TEXT,
    generated_at_epoch_ms INTEGER NOT NULL,
    imported_at_epoch_ms INTEGER,
    machine_count INTEGER NOT NULL DEFAULT 0 CHECK (machine_count >= 0),
    active INTEGER NOT NULL DEFAULT 0 CHECK (active IN (0, 1))
);
CREATE UNIQUE INDEX metadata_generation_one_active
    ON metadata_generation(active)
    WHERE active = 1;
CREATE INDEX metadata_generation_identity
    ON metadata_generation(executable_path, raw_version_line);

CREATE TABLE machines (
    generation_id INTEGER NOT NULL,
    short_name TEXT NOT NULL,
    description TEXT NOT NULL,
    year TEXT,
    manufacturer TEXT,
    source_file TEXT,
    clone_of TEXT,
    rom_of TEXT,
    is_bios INTEGER NOT NULL CHECK (is_bios IN (0, 1)),
    is_device INTEGER NOT NULL CHECK (is_device IN (0, 1)),
    is_mechanical INTEGER NOT NULL CHECK (is_mechanical IN (0, 1)),
    runnable INTEGER NOT NULL CHECK (runnable IN (0, 1)),
    driver_status TEXT,
    driver_emulation TEXT,
    driver_cocktail TEXT,
    driver_savestate TEXT,
    driver_requires_artwork INTEGER NOT NULL DEFAULT 0 CHECK (driver_requires_artwork IN (0, 1)),
    driver_unofficial INTEGER NOT NULL DEFAULT 0 CHECK (driver_unofficial IN (0, 1)),
    driver_no_sound_hardware INTEGER NOT NULL DEFAULT 0 CHECK (driver_no_sound_hardware IN (0, 1)),
    driver_incomplete INTEGER NOT NULL DEFAULT 0 CHECK (driver_incomplete IN (0, 1)),
    PRIMARY KEY (generation_id, short_name),
    FOREIGN KEY (generation_id) REFERENCES metadata_generation(id) ON DELETE CASCADE
);
CREATE INDEX machines_generation_description
    ON machines(generation_id, description COLLATE NOCASE);
CREATE INDEX machines_generation_short_name
    ON machines(generation_id, short_name COLLATE NOCASE);
CREATE INDEX machines_generation_manufacturer
    ON machines(generation_id, manufacturer COLLATE NOCASE);
CREATE INDEX machines_generation_year
    ON machines(generation_id, year);
CREATE INDEX machines_generation_status
    ON machines(generation_id, driver_status);
CREATE INDEX machines_generation_clone
    ON machines(generation_id, clone_of);

CREATE TABLE chips (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    generation_id INTEGER NOT NULL,
    machine_short_name TEXT NOT NULL,
    chip_type TEXT NOT NULL,
    name TEXT NOT NULL,
    tag TEXT,
    clock_hz INTEGER,
    FOREIGN KEY (generation_id, machine_short_name)
        REFERENCES machines(generation_id, short_name) ON DELETE CASCADE
);
CREATE INDEX chips_machine
    ON chips(generation_id, machine_short_name);

CREATE TABLE displays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    generation_id INTEGER NOT NULL,
    machine_short_name TEXT NOT NULL,
    tag TEXT,
    display_type TEXT NOT NULL,
    rotate INTEGER,
    flip_x INTEGER NOT NULL CHECK (flip_x IN (0, 1)),
    width INTEGER,
    height INTEGER,
    refresh_hz REAL NOT NULL,
    pixel_clock_hz INTEGER,
    FOREIGN KEY (generation_id, machine_short_name)
        REFERENCES machines(generation_id, short_name) ON DELETE CASCADE
);
CREATE INDEX displays_machine
    ON displays(generation_id, machine_short_name);

CREATE TABLE devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    generation_id INTEGER NOT NULL,
    machine_short_name TEXT NOT NULL,
    device_type TEXT NOT NULL,
    tag TEXT,
    fixed_image TEXT,
    mandatory TEXT,
    interface TEXT,
    instance_name TEXT,
    instance_brief_name TEXT,
    FOREIGN KEY (generation_id, machine_short_name)
        REFERENCES machines(generation_id, short_name) ON DELETE CASCADE
);
CREATE INDEX devices_machine
    ON devices(generation_id, machine_short_name);

CREATE TABLE device_extensions (
    device_id INTEGER NOT NULL,
    extension TEXT NOT NULL,
    PRIMARY KEY (device_id, extension),
    FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE
);

CREATE TABLE software_lists (
    generation_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY (generation_id, name),
    FOREIGN KEY (generation_id) REFERENCES metadata_generation(id) ON DELETE CASCADE
);

CREATE TABLE machine_software_lists (
    generation_id INTEGER NOT NULL,
    machine_short_name TEXT NOT NULL,
    tag TEXT NOT NULL,
    software_list_name TEXT NOT NULL,
    status TEXT NOT NULL,
    filter TEXT,
    PRIMARY KEY (generation_id, machine_short_name, tag, software_list_name),
    FOREIGN KEY (generation_id, machine_short_name)
        REFERENCES machines(generation_id, short_name) ON DELETE CASCADE,
    FOREIGN KEY (generation_id, software_list_name)
        REFERENCES software_lists(generation_id, name) ON DELETE CASCADE
);
CREATE INDEX machine_software_lists_machine
    ON machine_software_lists(generation_id, machine_short_name);

-- User-owned state intentionally has no foreign key into generated metadata.
-- A MAME upgrade may remove or rename a machine, but regeneration must never
-- cascade-delete the user's state.
CREATE TABLE user_favorites (
    machine_short_name TEXT PRIMARY KEY,
    created_at_epoch_ms INTEGER NOT NULL
);

CREATE TABLE user_collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    created_at_epoch_ms INTEGER NOT NULL,
    updated_at_epoch_ms INTEGER NOT NULL
);

CREATE TABLE user_collection_machines (
    collection_id INTEGER NOT NULL,
    machine_short_name TEXT NOT NULL,
    added_at_epoch_ms INTEGER NOT NULL,
    PRIMARY KEY (collection_id, machine_short_name),
    FOREIGN KEY (collection_id) REFERENCES user_collections(id) ON DELETE CASCADE
);

CREATE TABLE recent_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    machine_short_name TEXT NOT NULL,
    software_item TEXT,
    launched_at_epoch_ms INTEGER NOT NULL,
    succeeded INTEGER CHECK (succeeded IN (0, 1))
);
CREATE INDEX recent_history_launched_at
    ON recent_history(launched_at_epoch_ms DESC);

CREATE TABLE user_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE
);

CREATE TABLE user_machine_tags (
    tag_id INTEGER NOT NULL,
    machine_short_name TEXT NOT NULL,
    PRIMARY KEY (tag_id, machine_short_name),
    FOREIGN KEY (tag_id) REFERENCES user_tags(id) ON DELETE CASCADE
);
"#;

const CREATE_SCHEMA_V2: &str = r#"
CREATE TABLE machine_audit_results (
    machine_short_name TEXT PRIMARY KEY,
    classification TEXT NOT NULL CHECK (
        classification IN ('complete', 'bestAvailable', 'missingRequired', 'incorrect', 'mixedFailure', 'unknown')
    ),
    result_json TEXT NOT NULL,
    audited_at_epoch_ms INTEGER NOT NULL CHECK (audited_at_epoch_ms >= 0),
    mame_identity_json TEXT NOT NULL,
    content_paths_json TEXT NOT NULL
);
CREATE INDEX machine_audit_results_classification
    ON machine_audit_results(classification);
"#;

const CREATE_SCHEMA_V3: &str = r#"
-- User-owned per-machine launch preferences intentionally have no foreign key
-- into generated metadata. Metadata regeneration must not erase user overrides.
CREATE TABLE machine_launch_preferences (
    machine_short_name TEXT PRIMARY KEY,
    preferences_json TEXT NOT NULL
);
"#;

const CREATE_SCHEMA_V4: &str = r#"
-- Controller profiles are user-owned state. Device identity and mapping
-- provenance are typed JSON so future backends can preserve source-specific
-- identity without flattening it into an assumed hardware serial.
CREATE TABLE controller_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    device_identity_json TEXT NOT NULL,
    mapping_provenance_json TEXT NOT NULL,
    created_at_epoch_ms INTEGER NOT NULL CHECK (created_at_epoch_ms >= 0),
    updated_at_epoch_ms INTEGER NOT NULL CHECK (updated_at_epoch_ms >= 0)
);

-- Assignments intentionally do not reference generated machine metadata.
-- A metadata refresh must not erase a user's machine-scoped profile choice.
CREATE TABLE controller_profile_assignments (
    profile_id INTEGER NOT NULL,
    scope_kind TEXT NOT NULL CHECK (scope_kind IN ('global', 'machine')),
    scope_key TEXT NOT NULL,
    machine_short_name TEXT,
    CHECK (
        (scope_kind = 'global' AND scope_key = '*' AND machine_short_name IS NULL)
        OR
        (scope_kind = 'machine' AND machine_short_name IS NOT NULL AND scope_key = machine_short_name)
    ),
    PRIMARY KEY (profile_id, scope_kind, scope_key),
    FOREIGN KEY (profile_id) REFERENCES controller_profiles(id) ON DELETE CASCADE
);
CREATE INDEX controller_profile_assignments_scope
    ON controller_profile_assignments(scope_kind, scope_key);
"#;

fn database_error(code: &str, error: rusqlite::Error) -> AppError {
    AppError::new(code, "The catalog database operation failed.")
        .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{current_version, migrate, migrate_v0_to_v1, CATALOG_SCHEMA_VERSION};

    #[test]
    fn creates_current_schema_and_required_domains() {
        let mut connection = Connection::open_in_memory().expect("in-memory SQLite");
        migrate(&mut connection).expect("schema migration must succeed");

        assert_eq!(
            current_version(&connection).expect("schema version"),
            CATALOG_SCHEMA_VERSION
        );
        for table in [
            "metadata_generation",
            "machines",
            "displays",
            "devices",
            "software_lists",
            "machine_software_lists",
            "machine_audit_results",
            "machine_launch_preferences",
            "controller_profiles",
            "controller_profile_assignments",
            "user_favorites",
            "user_collections",
            "recent_history",
            "user_tags",
        ] {
            let exists: i64 = connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
                    [table],
                    |row| row.get(0),
                )
                .expect("table lookup");
            assert_eq!(exists, 1, "missing table {table}");
        }
    }

    #[test]
    fn migrates_existing_schema_without_losing_existing_user_state() {
        let mut connection = Connection::open_in_memory().expect("in-memory SQLite");
        migrate_v0_to_v1(&mut connection).expect("create v1 schema");
        assert_eq!(current_version(&connection).expect("v1 version"), 1);
        connection
            .execute(
                "INSERT INTO user_favorites(machine_short_name, created_at_epoch_ms) VALUES ('pacman', 1)",
                [],
            )
            .expect("seed v1 user state");

        migrate(&mut connection).expect("migrate v1 to current schema");

        assert_eq!(
            current_version(&connection).expect("current version"),
            CATALOG_SCHEMA_VERSION
        );
        let favorite_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM user_favorites", [], |row| row.get(0))
            .expect("favorite count");
        assert_eq!(favorite_count, 1, "v1 user state must survive migration");
        let audit_table_exists: i64 = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='machine_audit_results')",
                [],
                |row| row.get(0),
            )
            .expect("audit table lookup");
        assert_eq!(audit_table_exists, 1);
        let machine_settings_table_exists: i64 = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='machine_launch_preferences')",
                [],
                |row| row.get(0),
            )
            .expect("machine settings table lookup");
        assert_eq!(machine_settings_table_exists, 1);
    }

    #[test]
    fn future_schema_is_rejected_without_guessing() {
        let mut connection = Connection::open_in_memory().expect("in-memory SQLite");
        connection
            .execute_batch(
                "CREATE TABLE app_schema_version (singleton INTEGER PRIMARY KEY, version INTEGER NOT NULL);\nINSERT INTO app_schema_version VALUES (1, 999);",
            )
            .expect("seed future schema");

        let error = migrate(&mut connection).expect_err("future schema must fail");
        assert_eq!(error.code, "CATALOG_SCHEMA_UNSUPPORTED");
        assert_eq!(
            current_version(&connection).expect("version unchanged"),
            999
        );
    }

    #[test]
    fn failed_migration_rolls_back_partial_schema_changes() {
        let mut connection = Connection::open_in_memory().expect("in-memory SQLite");
        connection
            .execute_batch(
                "CREATE TABLE app_schema_version (singleton INTEGER PRIMARY KEY CHECK(singleton=1), version INTEGER NOT NULL);\nINSERT INTO app_schema_version VALUES (1, 0);",
            )
            .expect("seed intentionally conflicting v0 schema");

        let error = migrate(&mut connection).expect_err("conflicting migration must fail");
        assert_eq!(error.code, "CATALOG_SCHEMA_MIGRATION_FAILED");
        assert_eq!(current_version(&connection).expect("version unchanged"), 0);

        let machines_exists: i64 = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='machines')",
                [],
                |row| row.get(0),
            )
            .expect("machines lookup");
        assert_eq!(machines_exists, 0, "failed migration must roll back");
    }
}
