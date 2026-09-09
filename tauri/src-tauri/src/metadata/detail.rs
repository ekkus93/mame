use rusqlite::{params, OptionalExtension};

use crate::errors::{AppError, AppResult};

use super::{
    catalog::CatalogRepository,
    model::{MachineDetail, MachineDisplayInfo, MachineSoftwareListInfo},
};

impl CatalogRepository {
    pub(crate) fn machine_detail(&self, short_name: &str) -> AppResult<MachineDetail> {
        let generation_id: i64 = self
            .connection
            .query_row(
                "SELECT id FROM metadata_generation WHERE active = 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| database_error("CATALOG_DETAIL_QUERY_FAILED", error))?
            .ok_or_else(|| {
                AppError::new(
                    "MAME_METADATA_NOT_READY",
                    "No successfully imported MAME metadata generation is active.",
                )
            })?;

        let stored = self
            .connection
            .query_row(
                r#"SELECT
          m.short_name,
          m.description,
          m.year,
          m.manufacturer,
          m.source_file,
          m.clone_of,
          parent.description,
          m.rom_of,
          m.is_bios,
          m.is_device,
          m.is_mechanical,
          m.runnable,
          m.driver_status,
          m.driver_emulation,
          m.driver_cocktail,
          m.driver_savestate,
          m.driver_requires_artwork,
          m.driver_unofficial,
          m.driver_no_sound_hardware,
          m.driver_incomplete
      FROM machines m
      LEFT JOIN machines parent
        ON parent.generation_id = m.generation_id
       AND parent.short_name = m.clone_of
      WHERE m.generation_id = ?1 AND m.short_name = ?2"#,
                params![generation_id, short_name],
                stored_machine_detail_from_row,
            )
            .optional()
            .map_err(|error| database_error("CATALOG_DETAIL_QUERY_FAILED", error))?
            .ok_or_else(|| {
                AppError::new(
                    "CATALOG_MACHINE_NOT_FOUND",
                    "The requested machine is not present in the active MAME catalog.",
                )
                .with_details(serde_json::json!({ "shortName": short_name }))
            })?;

        let mut statement = self
            .connection
            .prepare(
                r#"SELECT tag, display_type, rotate, flip_x, width, height, refresh_hz, pixel_clock_hz
                   FROM displays
                   WHERE generation_id = ?1 AND machine_short_name = ?2
                   ORDER BY id"#,
            )
            .map_err(|error| database_error("CATALOG_DETAIL_QUERY_FAILED", error))?;
        let rows = statement
            .query_map(params![generation_id, short_name], stored_display_from_row)
            .map_err(|error| database_error("CATALOG_DETAIL_QUERY_FAILED", error))?;
        let mut displays = Vec::new();
        for row in rows {
            displays.push(
                row.map_err(|error| database_error("CATALOG_DETAIL_QUERY_FAILED", error))?
                    .into_info()?,
            );
        }

        let mut statement = self
            .connection
            .prepare(
                r#"SELECT tag, software_list_name, status, filter
                   FROM machine_software_lists
                   WHERE generation_id = ?1 AND machine_short_name = ?2
                   ORDER BY software_list_name COLLATE NOCASE ASC, tag COLLATE NOCASE ASC"#,
            )
            .map_err(|error| database_error("CATALOG_DETAIL_QUERY_FAILED", error))?;
        let rows = statement
            .query_map(params![generation_id, short_name], |row| {
                Ok(MachineSoftwareListInfo {
                    tag: row.get(0)?,
                    name: row.get(1)?,
                    status: row.get(2)?,
                    filter: row.get(3)?,
                })
            })
            .map_err(|error| database_error("CATALOG_DETAIL_QUERY_FAILED", error))?;
        let mut software_lists = Vec::new();
        for row in rows {
            software_lists
                .push(row.map_err(|error| database_error("CATALOG_DETAIL_QUERY_FAILED", error))?);
        }

        stored.into_detail(generation_id, displays, software_lists)
    }
}

#[derive(Debug)]
struct StoredMachineDetail {
    short_name: String,
    description: String,
    year: Option<String>,
    manufacturer: Option<String>,
    source_file: Option<String>,
    clone_of: Option<String>,
    parent_description: Option<String>,
    rom_of: Option<String>,
    is_bios: i64,
    is_device: i64,
    is_mechanical: i64,
    runnable: i64,
    driver_status: Option<String>,
    driver_emulation: Option<String>,
    driver_cocktail: Option<String>,
    driver_savestate: Option<String>,
    driver_requires_artwork: i64,
    driver_unofficial: i64,
    driver_no_sound_hardware: i64,
    driver_incomplete: i64,
}

impl StoredMachineDetail {
    fn into_detail(
        self,
        generation_id: i64,
        displays: Vec<MachineDisplayInfo>,
        software_lists: Vec<MachineSoftwareListInfo>,
    ) -> AppResult<MachineDetail> {
        Ok(MachineDetail {
            schema_version: 1,
            generation_id,
            short_name: self.short_name,
            description: self.description,
            year: self.year,
            manufacturer: self.manufacturer,
            source_file: self.source_file,
            clone_of: self.clone_of,
            parent_description: self.parent_description,
            rom_of: self.rom_of,
            is_bios: sqlite_bool(self.is_bios, "isBios")?,
            is_device: sqlite_bool(self.is_device, "isDevice")?,
            is_mechanical: sqlite_bool(self.is_mechanical, "isMechanical")?,
            runnable: sqlite_bool(self.runnable, "runnable")?,
            driver_status: self.driver_status,
            driver_emulation: self.driver_emulation,
            driver_cocktail: self.driver_cocktail,
            driver_savestate: self.driver_savestate,
            driver_requires_artwork: sqlite_bool(
                self.driver_requires_artwork,
                "driverRequiresArtwork",
            )?,
            driver_unofficial: sqlite_bool(self.driver_unofficial, "driverUnofficial")?,
            driver_no_sound_hardware: sqlite_bool(
                self.driver_no_sound_hardware,
                "driverNoSoundHardware",
            )?,
            driver_incomplete: sqlite_bool(self.driver_incomplete, "driverIncomplete")?,
            displays,
            software_lists,
        })
    }
}

fn stored_machine_detail_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<StoredMachineDetail> {
    Ok(StoredMachineDetail {
        short_name: row.get(0)?,
        description: row.get(1)?,
        year: row.get(2)?,
        manufacturer: row.get(3)?,
        source_file: row.get(4)?,
        clone_of: row.get(5)?,
        parent_description: row.get(6)?,
        rom_of: row.get(7)?,
        is_bios: row.get(8)?,
        is_device: row.get(9)?,
        is_mechanical: row.get(10)?,
        runnable: row.get(11)?,
        driver_status: row.get(12)?,
        driver_emulation: row.get(13)?,
        driver_cocktail: row.get(14)?,
        driver_savestate: row.get(15)?,
        driver_requires_artwork: row.get(16)?,
        driver_unofficial: row.get(17)?,
        driver_no_sound_hardware: row.get(18)?,
        driver_incomplete: row.get(19)?,
    })
}

#[derive(Debug)]
struct StoredDisplay {
    tag: Option<String>,
    display_type: String,
    rotate: Option<i64>,
    flip_x: i64,
    width: Option<i64>,
    height: Option<i64>,
    refresh_hz: f64,
    pixel_clock_hz: Option<i64>,
}

impl StoredDisplay {
    fn into_info(self) -> AppResult<MachineDisplayInfo> {
        Ok(MachineDisplayInfo {
            tag: self.tag,
            display_type: self.display_type,
            rotate: self.rotate,
            flip_x: sqlite_bool(self.flip_x, "display.flipX")?,
            width: self.width,
            height: self.height,
            refresh_hz: self.refresh_hz,
            pixel_clock_hz: self.pixel_clock_hz,
        })
    }
}

fn stored_display_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredDisplay> {
    Ok(StoredDisplay {
        tag: row.get(0)?,
        display_type: row.get(1)?,
        rotate: row.get(2)?,
        flip_x: row.get(3)?,
        width: row.get(4)?,
        height: row.get(5)?,
        refresh_hz: row.get(6)?,
        pixel_clock_hz: row.get(7)?,
    })
}

fn sqlite_bool(value: i64, field: &str) -> AppResult<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        invalid => Err(AppError::new(
            "CATALOG_DETAIL_RESULT_INVALID",
            "The catalog contains an invalid boolean value.",
        )
        .with_details(serde_json::json!({ "field": field, "value": invalid }))),
    }
}

fn database_error(code: &str, error: rusqlite::Error) -> AppError {
    AppError::new(code, "The machine detail database operation failed.")
        .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::mame::{MameExecutableIdentity, MameExecutableSourceKind, MameExecutableTrust};

    use super::CatalogRepository;
    use crate::metadata::{CloneFilter, MachineQuery, MachineSort};

    const FIXTURE: &str = include_str!("../../../tests/fixtures/listxml-representative.xml");

    fn identity() -> MameExecutableIdentity {
        MameExecutableIdentity {
            source: MameExecutableSourceKind::External,
            trust: MameExecutableTrust::UserConfigured,
            path: "/opt/mame/mame".to_owned(),
            version: "0.288".to_owned(),
            build: Some("test-fixture".to_owned()),
            raw_version_line: "0.288 test-fixture".to_owned(),
        }
    }

    fn repository() -> CatalogRepository {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let mut import = repository
            .begin_import(&identity(), 100)
            .expect("begin fixture import");
        let summary = import
            .import_listxml(Cursor::new(FIXTURE.as_bytes()))
            .expect("parse fixture");
        import.finish(summary, 200).expect("finish fixture import");
        repository
    }

    #[test]
    fn detail_includes_relationship_driver_and_display_metadata() {
        let repository = repository();
        let detail = repository
            .machine_detail("galaxiana")
            .expect("clone detail");
        assert_eq!(detail.clone_of.as_deref(), Some("galaxian"));
        assert_eq!(
            detail.parent_description.as_deref(),
            Some("Galaxian (Namco set 1)")
        );
        assert_eq!(detail.driver_status.as_deref(), Some("good"));
        assert_eq!(detail.displays.len(), 1);
        assert_eq!(detail.displays[0].rotate, Some(90));
    }

    #[test]
    fn detail_exposes_machine_software_list_associations() {
        let repository = repository();
        let detail = repository.machine_detail("apple2e").expect("apple detail");
        assert_eq!(detail.software_lists.len(), 2);
        assert_eq!(detail.software_lists[0].name, "apple2_flop_clcracked");
        assert_eq!(detail.software_lists[1].name, "apple2_flop_orig");
        assert_eq!(detail.software_lists[1].status, "original");
        assert_eq!(detail.software_lists[1].filter.as_deref(), Some("A2"));
    }

    #[test]
    fn machine_query_sort_is_applied_before_pagination() {
        let repository = repository();
        let page = repository
            .query_machines(&MachineQuery {
                text: None,
                manufacturer: None,
                year: None,
                driver_status: None,
                clone_filter: CloneFilter::All,
                sort: MachineSort::YearDesc,
                include_devices: false,
                limit: 2,
                offset: 0,
            })
            .expect("sorted query");
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[0].short_name, "apple2e");
        assert!(page.items[1].year.as_deref() <= page.items[0].year.as_deref());
    }

    #[test]
    fn missing_machine_detail_is_explicit() {
        let repository = repository();
        let error = repository
            .machine_detail("missing")
            .expect_err("missing machine must fail");
        assert_eq!(error.code, "CATALOG_MACHINE_NOT_FOUND");
    }
}
