use std::collections::BTreeMap;

use rusqlite::{named_params, OptionalExtension};

use crate::errors::{AppError, AppResult};

use super::{
    catalog::CatalogRepository,
    model::{MachineAvailability, MachineListItem, MachinePage},
};

const MAX_MAME_UI_PAGE_SIZE: u32 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MameUiMachineFilter {
    All,
    Available,
    Unavailable,
    Working,
    NotWorking,
    Mechanical,
    NotMechanical,
    Favorites,
    Bios,
    NotBios,
    Parents,
    Clones,
    Manufacturer,
    Year,
    SourceFile,
    SaveSupported,
    SaveUnsupported,
    VerticalScreen,
    HorizontalScreen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MameUiMachineQuery {
    pub text: Option<String>,
    pub filter: MameUiMachineFilter,
    pub filter_value: Option<String>,
    pub preferred_machine: Option<String>,
    pub limit: u32,
    pub offset: u32,
    pub audit_identity_json: Option<String>,
    pub audit_content_paths_json: Option<String>,
}

impl CatalogRepository {
    pub(crate) fn query_mame_ui_machines(
        &self,
        query: &MameUiMachineQuery,
    ) -> AppResult<MachinePage> {
        if query.limit == 0 || query.limit > MAX_MAME_UI_PAGE_SIZE {
            return Err(AppError::new(
                "CATALOG_QUERY_LIMIT_INVALID",
                format!(
                    "MAME UI catalog query limit must be between 1 and {MAX_MAME_UI_PAGE_SIZE}."
                ),
            ));
        }

        let generation_id: i64 = self
            .connection
            .query_row(
                "SELECT id FROM metadata_generation WHERE active = 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(database_error)?
            .ok_or_else(|| {
                AppError::new(
                    "MAME_METADATA_NOT_READY",
                    "No successfully imported MAME metadata generation is active.",
                )
            })?;

        let search_pattern = query.text.as_deref().map(like_pattern);
        let predicate = filter_predicate(query.filter);
        let sql_where = format!(
            r#"
                m.generation_id = :generation_id
                AND m.is_device = 0
                AND (
                    :search_pattern IS NULL
                    OR m.short_name LIKE :search_pattern ESCAPE '\\' COLLATE NOCASE
                    OR m.description LIKE :search_pattern ESCAPE '\\' COLLATE NOCASE
                    OR m.manufacturer LIKE :search_pattern ESCAPE '\\' COLLATE NOCASE
                )
                AND (:filter_value IS NULL OR :filter_value IS NOT NULL)
                AND ({predicate})
            "#
        );
        let audit_join = r#"
            LEFT JOIN machine_audit_results ar
              ON ar.machine_short_name = m.short_name
             AND ar.mame_identity_json = :audit_identity
             AND ar.content_paths_json = :audit_paths
        "#;

        let total_sql = format!("SELECT COUNT(*) FROM machines m {audit_join} WHERE {sql_where}");
        let total_i64: i64 = self
            .connection
            .query_row(
                &total_sql,
                named_params! {
                    ":generation_id": generation_id,
                    ":search_pattern": search_pattern.as_deref(),
                    ":filter_value": query.filter_value.as_deref(),
                    ":audit_identity": query.audit_identity_json.as_deref(),
                    ":audit_paths": query.audit_content_paths_json.as_deref(),
                },
                |row| row.get(0),
            )
            .map_err(database_error)?;
        let total = u64::try_from(total_i64).map_err(|_| {
            AppError::new(
                "CATALOG_QUERY_RESULT_INVALID",
                "The catalog returned an invalid negative result count.",
            )
        })?;

        let effective_offset = if query.offset == 0 {
            self.preferred_page_offset(
                query,
                generation_id,
                search_pattern.as_deref(),
                audit_join,
                &sql_where,
            )?
            .unwrap_or(query.offset)
        } else {
            query.offset
        };

        let sql = format!(
            r#"
            SELECT
                m.short_name,
                m.description,
                m.year,
                m.manufacturer,
                m.source_file,
                m.clone_of,
                m.runnable,
                m.is_device,
                m.driver_status,
                CASE
                    WHEN ar.classification IN ('complete', 'bestAvailable') THEN 'available'
                    WHEN ar.classification IN ('missingRequired', 'incorrect', 'mixedFailure') THEN 'missing'
                    ELSE 'unknown'
                END AS availability,
                (SELECT COUNT(*) FROM displays d
                    WHERE d.generation_id = m.generation_id
                      AND d.machine_short_name = m.short_name) AS display_count,
                (SELECT COUNT(*) FROM machine_software_lists sl
                    WHERE sl.generation_id = m.generation_id
                      AND sl.machine_short_name = m.short_name) AS software_list_count
            FROM machines m
            {audit_join}
            WHERE {sql_where}
            ORDER BY m.description COLLATE NOCASE ASC, m.short_name COLLATE NOCASE ASC
            LIMIT :limit OFFSET :offset
            "#
        );
        let mut statement = self.connection.prepare(&sql).map_err(database_error)?;
        let rows = statement
            .query_map(
                named_params! {
                    ":generation_id": generation_id,
                    ":search_pattern": search_pattern.as_deref(),
                    ":filter_value": query.filter_value.as_deref(),
                    ":audit_identity": query.audit_identity_json.as_deref(),
                    ":audit_paths": query.audit_content_paths_json.as_deref(),
                    ":limit": i64::from(query.limit),
                    ":offset": i64::from(effective_offset),
                },
                |row| {
                    Ok(StoredMameUiMachineItem {
                        short_name: row.get(0)?,
                        description: row.get(1)?,
                        year: row.get(2)?,
                        manufacturer: row.get(3)?,
                        source_file: row.get(4)?,
                        clone_of: row.get(5)?,
                        runnable: row.get(6)?,
                        is_device: row.get(7)?,
                        driver_status: row.get(8)?,
                        availability: row.get(9)?,
                        display_count: row.get(10)?,
                        software_list_count: row.get(11)?,
                    })
                },
            )
            .map_err(database_error)?;

        let mut items = Vec::new();
        let mut availability_by_short_name = BTreeMap::new();
        for row in rows {
            let stored = row.map_err(database_error)?;
            let availability = machine_availability_from_token(&stored.availability)?;
            let item = MachineListItem {
                short_name: stored.short_name,
                description: stored.description,
                year: stored.year,
                manufacturer: stored.manufacturer,
                source_file: stored.source_file,
                clone_of: stored.clone_of,
                runnable: sqlite_bool(stored.runnable, "runnable")?,
                is_device: sqlite_bool(stored.is_device, "isDevice")?,
                driver_status: stored.driver_status,
                display_count: to_u32(stored.display_count, "displayCount")?,
                software_list_count: to_u32(stored.software_list_count, "softwareListCount")?,
            };
            availability_by_short_name.insert(item.short_name.clone(), availability);
            items.push(item);
        }

        Ok(MachinePage {
            schema_version: 1,
            generation_id,
            total,
            offset: effective_offset,
            limit: query.limit,
            items,
            availability_by_short_name,
        })
    }

    fn preferred_page_offset(
        &self,
        query: &MameUiMachineQuery,
        generation_id: i64,
        search_pattern: Option<&str>,
        audit_join: &str,
        sql_where: &str,
    ) -> AppResult<Option<u32>> {
        let Some(preferred_machine) = query.preferred_machine.as_deref() else {
            return Ok(None);
        };

        let sql = format!(
            r#"
            WITH filtered AS (
                SELECT
                    m.short_name,
                    ROW_NUMBER() OVER (
                        ORDER BY m.description COLLATE NOCASE ASC, m.short_name COLLATE NOCASE ASC
                    ) - 1 AS row_position
                FROM machines m
                {audit_join}
                WHERE {sql_where}
            )
            SELECT row_position
            FROM filtered
            WHERE short_name = :preferred_machine
            LIMIT 1
            "#
        );
        let position: Option<i64> = self
            .connection
            .query_row(
                &sql,
                named_params! {
                    ":generation_id": generation_id,
                    ":search_pattern": search_pattern,
                    ":filter_value": query.filter_value.as_deref(),
                    ":audit_identity": query.audit_identity_json.as_deref(),
                    ":audit_paths": query.audit_content_paths_json.as_deref(),
                    ":preferred_machine": preferred_machine,
                },
                |row| row.get(0),
            )
            .optional()
            .map_err(database_error)?;

        position
            .map(|position| {
                let position = u64::try_from(position).map_err(|_| {
                    AppError::new(
                        "CATALOG_QUERY_RESULT_INVALID",
                        "The catalog returned an invalid preferred-machine position.",
                    )
                })?;
                let page_size = u64::from(query.limit);
                let offset = (position / page_size) * page_size;
                u32::try_from(offset).map_err(|_| {
                    AppError::new(
                        "CATALOG_QUERY_RESULT_INVALID",
                        "The preferred-machine page offset exceeds the supported query range.",
                    )
                })
            })
            .transpose()
    }
}

fn filter_predicate(filter: MameUiMachineFilter) -> &'static str {
    match filter {
        MameUiMachineFilter::All => "1 = 1",
        MameUiMachineFilter::Available => {
            "ar.classification IN ('complete', 'bestAvailable')"
        }
        MameUiMachineFilter::Unavailable => {
            "ar.classification IN ('missingRequired', 'incorrect', 'mixedFailure')"
        }
        MameUiMachineFilter::Working => {
            "m.driver_status IS NOT NULL AND m.driver_status <> 'preliminary'"
        }
        MameUiMachineFilter::NotWorking => "m.driver_status = 'preliminary'",
        MameUiMachineFilter::Mechanical => "m.is_mechanical = 1",
        MameUiMachineFilter::NotMechanical => "m.is_mechanical = 0",
        MameUiMachineFilter::Favorites => {
            "EXISTS (SELECT 1 FROM user_favorites uf WHERE uf.machine_short_name = m.short_name)"
        }
        MameUiMachineFilter::Bios => "m.is_bios = 1",
        MameUiMachineFilter::NotBios => "m.is_bios = 0",
        MameUiMachineFilter::Parents => {
            "m.clone_of IS NULL OR EXISTS (SELECT 1 FROM machines p WHERE p.generation_id = m.generation_id AND p.short_name = m.clone_of AND p.is_bios = 1)"
        }
        MameUiMachineFilter::Clones => {
            "m.clone_of IS NOT NULL AND NOT EXISTS (SELECT 1 FROM machines p WHERE p.generation_id = m.generation_id AND p.short_name = m.clone_of AND p.is_bios = 1)"
        }
        MameUiMachineFilter::Manufacturer => {
            "(:filter_value IS NOT NULL AND (m.manufacturer = :filter_value COLLATE NOCASE OR (instr(m.manufacturer, ' (') > 0 AND substr(m.manufacturer, 1, instr(m.manufacturer, ' (') - 1) = :filter_value COLLATE NOCASE)))"
        }
        MameUiMachineFilter::Year => ":filter_value IS NOT NULL AND m.year = :filter_value",
        MameUiMachineFilter::SourceFile => {
            ":filter_value IS NOT NULL AND m.source_file = :filter_value COLLATE NOCASE"
        }
        MameUiMachineFilter::SaveSupported => {
            "m.driver_savestate IS NOT NULL AND m.driver_savestate <> 'unsupported'"
        }
        MameUiMachineFilter::SaveUnsupported => {
            "m.driver_savestate IS NULL OR m.driver_savestate = 'unsupported'"
        }
        MameUiMachineFilter::VerticalScreen => {
            "EXISTS (SELECT 1 FROM displays d WHERE d.generation_id = m.generation_id AND d.machine_short_name = m.short_name AND d.rotate IN (90, 270))"
        }
        MameUiMachineFilter::HorizontalScreen => {
            "EXISTS (SELECT 1 FROM displays d WHERE d.generation_id = m.generation_id AND d.machine_short_name = m.short_name) AND NOT EXISTS (SELECT 1 FROM displays d WHERE d.generation_id = m.generation_id AND d.machine_short_name = m.short_name AND d.rotate IN (90, 270))"
        }
    }
}

#[derive(Debug)]
struct StoredMameUiMachineItem {
    short_name: String,
    description: String,
    year: Option<String>,
    manufacturer: Option<String>,
    source_file: Option<String>,
    clone_of: Option<String>,
    runnable: i64,
    is_device: i64,
    driver_status: Option<String>,
    availability: String,
    display_count: i64,
    software_list_count: i64,
}

fn like_pattern(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('%');
    for character in value.chars() {
        match character {
            '\\' | '%' | '_' => {
                escaped.push('\\');
                escaped.push(character);
            }
            other => escaped.push(other),
        }
    }
    escaped.push('%');
    escaped
}

fn sqlite_bool(value: i64, field: &str) -> AppResult<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        invalid => Err(AppError::new(
            "CATALOG_QUERY_RESULT_INVALID",
            "The catalog contains an invalid boolean value.",
        )
        .with_details(serde_json::json!({ "field": field, "value": invalid }))),
    }
}

fn to_u32(value: i64, field: &str) -> AppResult<u32> {
    u32::try_from(value).map_err(|_| {
        AppError::new(
            "CATALOG_QUERY_RESULT_INVALID",
            "The catalog contains an integer outside the supported result range.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn machine_availability_from_token(value: &str) -> AppResult<MachineAvailability> {
    match value {
        "available" => Ok(MachineAvailability::Available),
        "missing" => Ok(MachineAvailability::Missing),
        "unknown" => Ok(MachineAvailability::Unknown),
        invalid => Err(AppError::new(
            "CATALOG_QUERY_RESULT_INVALID",
            "The MAME UI catalog query produced an invalid availability value.",
        )
        .with_details(serde_json::json!({ "availability": invalid }))),
    }
}

fn database_error(error: rusqlite::Error) -> AppError {
    AppError::new(
        "CATALOG_QUERY_FAILED",
        "The catalog database operation failed.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use super::{filter_predicate, like_pattern, MameUiMachineFilter};

    #[test]
    fn every_canonical_filter_has_a_nonempty_predicate() {
        for filter in [
            MameUiMachineFilter::All,
            MameUiMachineFilter::Available,
            MameUiMachineFilter::Unavailable,
            MameUiMachineFilter::Working,
            MameUiMachineFilter::NotWorking,
            MameUiMachineFilter::Mechanical,
            MameUiMachineFilter::NotMechanical,
            MameUiMachineFilter::Favorites,
            MameUiMachineFilter::Bios,
            MameUiMachineFilter::NotBios,
            MameUiMachineFilter::Parents,
            MameUiMachineFilter::Clones,
            MameUiMachineFilter::Manufacturer,
            MameUiMachineFilter::Year,
            MameUiMachineFilter::SourceFile,
            MameUiMachineFilter::SaveSupported,
            MameUiMachineFilter::SaveUnsupported,
            MameUiMachineFilter::VerticalScreen,
            MameUiMachineFilter::HorizontalScreen,
        ] {
            assert!(!filter_predicate(filter).trim().is_empty());
        }
    }

    #[test]
    fn text_search_escapes_sql_like_metacharacters() {
        assert_eq!(like_pattern(r"a%b_c\d"), r"%a\%b\_c\\d%");
    }

    #[test]
    fn horizontal_filter_requires_a_display_and_excludes_rotated_displays() {
        let predicate = filter_predicate(MameUiMachineFilter::HorizontalScreen);
        assert!(predicate.contains("EXISTS"));
        assert!(predicate.contains("NOT EXISTS"));
    }
}
