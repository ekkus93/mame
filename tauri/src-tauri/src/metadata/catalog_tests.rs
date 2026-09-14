#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::mame::{MameExecutableIdentity, MameExecutableSourceKind, MameExecutableTrust};

    use super::{
        AvailabilityFilter, CatalogRepository, CloneFilter, MachineAvailabilityQuery, MachineQuery,
        MachineSort,
    };
    use crate::metadata::model::{MachineAvailability, MetadataFreshness};

    const FIXTURE: &str = include_str!("../../../tests/fixtures/listxml-representative.xml");

    #[test]
    fn import_activates_complete_generation_and_queries_it() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        import_fixture(&mut repository, &identity, FIXTURE, 100, 200)
            .expect("fixture import must succeed");

        let generation = repository
            .active_generation()
            .expect("active generation query")
            .expect("active generation");
        assert_eq!(generation.machine_count, 4);
        assert_eq!(
            generation.listxml_build.as_deref(),
            Some("0.288 test-fixture")
        );

        let page = repository
            .query_machines(&MachineQuery {
                text: Some("galax".to_owned()),
                manufacturer: None,
                year: None,
                driver_status: None,
                clone_filter: CloneFilter::All,
                sort: MachineSort::DescriptionAsc,
                include_devices: false,
                limit: 50,
                offset: 0,
            })
            .expect("search query");
        assert_eq!(page.total, 2);
        assert_eq!(page.items.len(), 2);
        assert!(page.items.iter().all(|item| !item.is_device));
    }

    #[test]
    fn availability_is_provenance_gated_and_filtered_before_pagination() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        import_fixture(&mut repository, &identity, FIXTURE, 100, 200)
            .expect("fixture import must succeed");

        for (machine, classification) in
            [("galaxian", "complete"), ("galaxiana", "missingRequired")]
        {
            repository
                .connection
                .execute(
                    r#"INSERT INTO machine_audit_results(
                        machine_short_name, classification, result_json, audited_at_epoch_ms,
                        mame_identity_json, content_paths_json
                    ) VALUES (?1, ?2, '{}', 123, 'identity', 'paths')"#,
                    rusqlite::params![machine, classification],
                )
                .expect("seed audit result");
        }

        let query = MachineQuery {
            text: Some("galax".to_owned()),
            manufacturer: None,
            year: None,
            driver_status: None,
            clone_filter: CloneFilter::All,
            sort: MachineSort::DescriptionAsc,
            include_devices: false,
            limit: 50,
            offset: 0,
        };
        let current = |filter| {
            MachineAvailabilityQuery::current(filter, "identity".to_owned(), "paths".to_owned())
        };

        let page = repository
            .query_machines_with_availability(&query, &current(AvailabilityFilter::All))
            .expect("availability query");
        assert_eq!(
            page.availability_by_short_name.get("galaxian"),
            Some(&MachineAvailability::Available)
        );
        assert_eq!(
            page.availability_by_short_name.get("galaxiana"),
            Some(&MachineAvailability::Missing)
        );

        let available = repository
            .query_machines_with_availability(&query, &current(AvailabilityFilter::Available))
            .expect("available filter");
        assert_eq!(available.total, 1);
        assert_eq!(available.items[0].short_name, "galaxian");

        let missing = repository
            .query_machines_with_availability(&query, &current(AvailabilityFilter::Missing))
            .expect("missing filter");
        assert_eq!(missing.total, 1);
        assert_eq!(missing.items[0].short_name, "galaxiana");

        let stale = MachineAvailabilityQuery::current(
            AvailabilityFilter::Unknown,
            "different-identity".to_owned(),
            "paths".to_owned(),
        );
        let unknown = repository
            .query_machines_with_availability(&query, &stale)
            .expect("stale provenance must fail closed");
        assert_eq!(unknown.total, 2);
        assert!(unknown
            .availability_by_short_name
            .values()
            .all(|status| *status == MachineAvailability::Unknown));
    }

    #[test]
    fn filters_parent_clone_year_manufacturer_and_status() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        import_fixture(&mut repository, &identity, FIXTURE, 100, 200)
            .expect("fixture import must succeed");

        let page = repository
            .query_machines(&MachineQuery {
                text: None,
                manufacturer: Some("Namco".to_owned()),
                year: Some("1979".to_owned()),
                driver_status: Some("good".to_owned()),
                clone_filter: CloneFilter::ClonesOnly,
                sort: MachineSort::DescriptionAsc,
                include_devices: false,
                limit: 25,
                offset: 0,
            })
            .expect("filtered query");
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].short_name, "galaxiana");
    }

    #[test]
    fn device_records_are_hidden_unless_explicitly_requested() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        import_fixture(&mut repository, &identity, FIXTURE, 100, 200)
            .expect("fixture import must succeed");

        let hidden = repository
            .query_machines(&MachineQuery {
                text: Some("Zilog Z80".to_owned()),
                manufacturer: None,
                year: None,
                driver_status: None,
                clone_filter: CloneFilter::All,
                sort: MachineSort::DescriptionAsc,
                include_devices: false,
                limit: 25,
                offset: 0,
            })
            .expect("hidden device query");
        assert_eq!(hidden.total, 0);

        let visible = repository
            .query_machines(&MachineQuery {
                include_devices: true,
                ..MachineQuery {
                    text: Some("Zilog Z80".to_owned()),
                    manufacturer: None,
                    year: None,
                    driver_status: None,
                    clone_filter: CloneFilter::All,
                    sort: MachineSort::DescriptionAsc,
                    include_devices: false,
                    limit: 25,
                    offset: 0,
                }
            })
            .expect("visible device query");
        assert_eq!(visible.total, 1);
        assert!(visible.items[0].is_device);
    }

    #[test]
    fn failed_replacement_import_preserves_previous_active_generation() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        let first =
            import_fixture(&mut repository, &identity, FIXTURE, 100, 200).expect("first import");

        let duplicate_xml = r#"<mame build="0.288 test-fixture" mameconfig="10">
          <machine name="dup"><description>One</description></machine>
          <machine name="dup"><description>Two</description></machine>
        </mame>"#;
        let mut replacement = repository
            .begin_import(&identity, 300)
            .expect("replacement transaction");
        let error = replacement
            .import_listxml(Cursor::new(duplicate_xml.as_bytes()))
            .expect_err("duplicate machine must fail import");
        assert_eq!(error.code, "CATALOG_METADATA_INSERT_FAILED");
        drop(replacement);

        let active = repository
            .active_generation()
            .expect("active generation query")
            .expect("old active generation retained");
        assert_eq!(active.generation_id, first.generation_id);
        assert_eq!(active.machine_count, 4);
    }

    #[test]
    fn regeneration_does_not_delete_user_owned_state() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        import_fixture(&mut repository, &identity, FIXTURE, 100, 200).expect("first import");
        repository
            .connection
            .execute(
                "INSERT INTO user_favorites(machine_short_name, created_at_epoch_ms) VALUES ('galaxian', 123)",
                [],
            )
            .expect("seed favorite");
        repository
            .connection
            .execute(
                "INSERT INTO user_collections(name, created_at_epoch_ms, updated_at_epoch_ms) VALUES ('Classics', 124, 125)",
                [],
            )
            .expect("seed collection");
        let collection_id = repository.connection.last_insert_rowid();
        repository
            .connection
            .execute(
                "INSERT INTO user_collection_machines(collection_id, machine_short_name, added_at_epoch_ms) VALUES (?1, 'galaxian', 126)",
                rusqlite::params![collection_id],
            )
            .expect("seed collection membership");
        repository
            .connection
            .execute(
                "INSERT INTO recent_history(machine_short_name, software_item, launched_at_epoch_ms, succeeded) VALUES ('galaxian', NULL, 127, 1)",
                [],
            )
            .expect("seed recent history");

        let replacement_xml = r#"<mame build="0.288 test-fixture" mameconfig="10">
          <machine name="newmachine"><description>New Machine</description></machine>
        </mame>"#;
        import_fixture(&mut repository, &identity, replacement_xml, 300, 400)
            .expect("replacement import");

        let favorite_count: i64 = repository
            .connection
            .query_row("SELECT COUNT(*) FROM user_favorites", [], |row| row.get(0))
            .expect("favorite count");
        assert_eq!(favorite_count, 1);
        let favorite: String = repository
            .connection
            .query_row("SELECT machine_short_name FROM user_favorites", [], |row| {
                row.get(0)
            })
            .expect("favorite value");
        assert_eq!(favorite, "galaxian");

        let collection_machine: String = repository
            .connection
            .query_row(
                "SELECT machine_short_name FROM user_collection_machines WHERE collection_id = ?1",
                rusqlite::params![collection_id],
                |row| row.get(0),
            )
            .expect("collection membership");
        assert_eq!(collection_machine, "galaxian");

        let recent_machine: String = repository
            .connection
            .query_row(
                "SELECT machine_short_name FROM recent_history ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("recent history");
        assert_eq!(recent_machine, "galaxian");
    }

    #[test]
    fn changed_executable_identity_marks_generation_stale() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let current_identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        import_fixture(&mut repository, &current_identity, FIXTURE, 100, 200)
            .expect("fixture import");

        let fresh = repository
            .metadata_status(current_identity.clone())
            .expect("freshness query");
        assert_eq!(fresh.freshness, MetadataFreshness::Fresh);

        let changed = identity("/opt/mame/mame-new", "0.289", "next-build");
        let stale = repository
            .metadata_status(changed)
            .expect("stale freshness query");
        assert_eq!(stale.freshness, MetadataFreshness::Stale);
    }

    #[test]
    fn query_limit_is_bounded() {
        let repository = CatalogRepository::memory().expect("catalog repository");
        let error = repository
            .query_machines(&MachineQuery {
                text: None,
                manufacturer: None,
                year: None,
                driver_status: None,
                clone_filter: CloneFilter::ParentsOnly,
                sort: MachineSort::DescriptionAsc,
                include_devices: false,
                limit: 201,
                offset: 0,
            })
            .expect_err("unbounded page must fail");
        assert_eq!(error.code, "CATALOG_QUERY_LIMIT_INVALID");
    }

    fn import_fixture(
        repository: &mut CatalogRepository,
        identity: &MameExecutableIdentity,
        xml: &str,
        generated_at: u64,
        imported_at: u64,
    ) -> crate::errors::AppResult<crate::metadata::model::MetadataGenerationSummary> {
        let mut import = repository.begin_import(identity, generated_at)?;
        let summary = import.import_listxml(Cursor::new(xml.as_bytes()))?;
        import.finish(summary, imported_at)
    }

    fn identity(path: &str, version: &str, build: &str) -> MameExecutableIdentity {
        MameExecutableIdentity {
            source: MameExecutableSourceKind::External,
            trust: MameExecutableTrust::UserConfigured,
            path: path.to_owned(),
            version: version.to_owned(),
            build: Some(build.to_owned()),
            raw_version_line: format!("{version} {build}"),
        }
    }
}
