pub(crate) struct CatalogImport<'connection> {
    transaction: Transaction<'connection>,
    generation_id: i64,
    identity: MameExecutableIdentity,
    generated_at_epoch_ms: u64,
    machine_count: u64,
}

impl CatalogImport<'_> {
    pub(crate) fn import_listxml<R: BufRead>(&mut self, reader: R) -> AppResult<ListXmlSummary> {
        parse_listxml(reader, |machine| self.insert_machine(&machine))
    }

    pub(crate) fn finish(
        self,
        listxml: ListXmlSummary,
        imported_at_epoch_ms: u64,
    ) -> AppResult<MetadataGenerationSummary> {
        if self.machine_count != listxml.machine_count {
            return Err(AppError::new(
                "CATALOG_IMPORT_COUNT_MISMATCH",
                "The parser and catalog importer disagree on the number of imported machines.",
            )
            .with_details(serde_json::json!({
                "parsed": listxml.machine_count,
                "inserted": self.machine_count
            })));
        }

        let imported_at = to_i64(imported_at_epoch_ms, "importedAtEpochMs")?;
        let machine_count = to_i64(self.machine_count, "machineCount")?;
        self.transaction
            .execute(
                r#"UPDATE metadata_generation
                   SET listxml_build = ?2,
                       mame_config = ?3,
                       imported_at_epoch_ms = ?4,
                       machine_count = ?5
                   WHERE id = ?1"#,
                params![
                    self.generation_id,
                    listxml.build,
                    listxml.mame_config,
                    imported_at,
                    machine_count
                ],
            )
            .map_err(|error| database_error("CATALOG_IMPORT_FINALIZE_FAILED", error))?;

        self.transaction
            .execute(
                "UPDATE metadata_generation SET active = 0 WHERE active = 1",
                [],
            )
            .map_err(|error| database_error("CATALOG_IMPORT_FINALIZE_FAILED", error))?;
        let activated = self
            .transaction
            .execute(
                "UPDATE metadata_generation SET active = 1 WHERE id = ?1",
                [self.generation_id],
            )
            .map_err(|error| database_error("CATALOG_IMPORT_FINALIZE_FAILED", error))?;
        if activated != 1 {
            return Err(AppError::new(
                "CATALOG_IMPORT_ACTIVATION_FAILED",
                "The completed metadata generation could not be activated.",
            ));
        }

        // Keep generated metadata bounded. User-owned tables do not reference a
        // generation, so pruning superseded generations cannot delete favorites,
        // collections, tags, or history.
        self.transaction
            .execute(
                "DELETE FROM metadata_generation WHERE active = 0 AND id <> ?1",
                [self.generation_id],
            )
            .map_err(|error| database_error("CATALOG_IMPORT_PRUNE_FAILED", error))?;

        let summary = MetadataGenerationSummary {
            generation_id: self.generation_id,
            source_kind: source_kind_token(self.identity.source).to_owned(),
            trust: trust_token(self.identity.trust).to_owned(),
            executable_path: self.identity.path.clone(),
            mame_version: self.identity.version.clone(),
            mame_build: self.identity.build.clone(),
            raw_version_line: self.identity.raw_version_line.clone(),
            listxml_build: listxml.build,
            mame_config: listxml.mame_config,
            generated_at_epoch_ms: self.generated_at_epoch_ms,
            imported_at_epoch_ms,
            machine_count: self.machine_count,
        };

        self.transaction
            .commit()
            .map_err(|error| database_error("CATALOG_IMPORT_COMMIT_FAILED", error))?;
        Ok(summary)
    }

    fn insert_machine(&mut self, machine: &MachineMetadata) -> AppResult<()> {
        let driver = machine.driver.as_ref();
        self.transaction
            .execute(
                r#"INSERT INTO machines(
                    generation_id, short_name, description, year, manufacturer, source_file,
                    clone_of, rom_of, is_bios, is_device, is_mechanical, runnable,
                    driver_status, driver_emulation, driver_cocktail, driver_savestate,
                    driver_requires_artwork, driver_unofficial, driver_no_sound_hardware,
                    driver_incomplete
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                    ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20
                )"#,
                params![
                    self.generation_id,
                    machine.short_name,
                    machine.description,
                    machine.year,
                    machine.manufacturer,
                    machine.source_file,
                    machine.clone_of,
                    machine.rom_of,
                    bool_i64(machine.is_bios),
                    bool_i64(machine.is_device),
                    bool_i64(machine.is_mechanical),
                    bool_i64(machine.runnable),
                    driver.map(|value| value.status.as_str()),
                    driver.map(|value| value.emulation.as_str()),
                    driver.and_then(|value| value.cocktail.as_deref()),
                    driver.map(|value| value.savestate.as_str()),
                    bool_i64(driver.is_some_and(|value| value.requires_artwork)),
                    bool_i64(driver.is_some_and(|value| value.unofficial)),
                    bool_i64(driver.is_some_and(|value| value.no_sound_hardware)),
                    bool_i64(driver.is_some_and(|value| value.incomplete)),
                ],
            )
            .map_err(|error| {
                AppError::new(
                    "CATALOG_METADATA_INSERT_FAILED",
                    "The machine metadata could not be inserted into the catalog.",
                )
                .with_details(serde_json::json!({
                    "machine": machine.short_name,
                    "cause": error.to_string()
                }))
            })?;

        if machine.requires_chd {
            self.transaction
                .execute(
                    "INSERT INTO machine_disk_presence(generation_id, machine_short_name) VALUES (?1, ?2)",
                    params![self.generation_id, machine.short_name],
                )
                .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
        }

        for chip in &machine.chips {
            let clock_hz = chip
                .clock_hz
                .map(|value| to_i64(value, "chipClockHz"))
                .transpose()?;
            self.transaction
                .execute(
                    r#"INSERT INTO chips(
                        generation_id, machine_short_name, chip_type, name, tag, clock_hz
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
                    params![
                        self.generation_id,
                        machine.short_name,
                        chip.chip_type,
                        chip.name,
                        chip.tag,
                        clock_hz
                    ],
                )
                .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
        }

        for display in &machine.displays {
            let pixel_clock_hz = display
                .pixel_clock_hz
                .map(|value| to_i64(value, "displayPixelClockHz"))
                .transpose()?;
            self.transaction
                .execute(
                    r#"INSERT INTO displays(
                        generation_id, machine_short_name, tag, display_type, rotate, flip_x,
                        width, height, refresh_hz, pixel_clock_hz
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
                    params![
                        self.generation_id,
                        machine.short_name,
                        display.tag,
                        display.display_type,
                        display.rotate.map(i64::from),
                        bool_i64(display.flip_x),
                        display.width.map(i64::from),
                        display.height.map(i64::from),
                        display.refresh_hz,
                        pixel_clock_hz
                    ],
                )
                .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
        }

        for device in &machine.devices {
            self.transaction
                .execute(
                    r#"INSERT INTO devices(
                        generation_id, machine_short_name, device_type, tag, fixed_image,
                        mandatory, interface, instance_name, instance_brief_name
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
                    params![
                        self.generation_id,
                        machine.short_name,
                        device.device_type,
                        device.tag,
                        device.fixed_image,
                        device.mandatory,
                        device.interface,
                        device.instance_name,
                        device.instance_brief_name
                    ],
                )
                .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
            let device_id = self.transaction.last_insert_rowid();
            for extension in &device.extensions {
                self.transaction
                    .execute(
                        "INSERT INTO device_extensions(device_id, extension) VALUES (?1, ?2)",
                        params![device_id, extension],
                    )
                    .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
            }
        }

        for software_list in &machine.software_lists {
            self.transaction
                .execute(
                    "INSERT OR IGNORE INTO software_lists(generation_id, name) VALUES (?1, ?2)",
                    params![self.generation_id, software_list.name],
                )
                .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
            self.transaction
                .execute(
                    r#"INSERT INTO machine_software_lists(
                        generation_id, machine_short_name, tag, software_list_name, status, filter
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
                    params![
                        self.generation_id,
                        machine.short_name,
                        software_list.tag,
                        software_list.name,
                        software_list.status,
                        software_list.filter
                    ],
                )
                .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
        }

        self.machine_count = self.machine_count.checked_add(1).ok_or_else(|| {
            AppError::new(
                "CATALOG_MACHINE_COUNT_OVERFLOW",
                "The imported catalog contains more machines than can be counted.",
            )
        })?;
        Ok(())
    }
}
