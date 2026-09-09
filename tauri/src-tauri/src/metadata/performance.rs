use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use rusqlite::params;

use super::{CatalogRepository, CloneFilter, MachineQuery, MachineSort};

const QUALIFICATION_MACHINE_COUNT: u32 = 100_000;
const QUALIFICATION_PAGE_SIZE: u32 = 100;
const STARTUP_QUERY_BUDGET: Duration = Duration::from_secs(2);
const SEARCH_QUERY_BUDGET: Duration = Duration::from_secs(1);
const TAIL_PAGE_QUERY_BUDGET: Duration = Duration::from_secs(2);
const SEARCH_SAMPLE_COUNT: usize = 5;

#[test]
#[ignore = "MT-410 full-catalog performance qualification; run explicitly in CI"]
fn full_catalog_library_queries_meet_interactive_budgets() {
    let catalog = TempCatalog::new();
    seed_catalog(catalog.path(), QUALIFICATION_MACHINE_COUNT);

    let startup_started = Instant::now();
    let repository = CatalogRepository::open(catalog.path()).expect("reopen seeded catalog");
    let startup_page = repository
        .query_machines(&query(None, 0))
        .expect("load first library page");
    let startup_elapsed = startup_started.elapsed();

    assert_eq!(startup_page.total, u64::from(QUALIFICATION_MACHINE_COUNT));
    assert_eq!(startup_page.items.len(), QUALIFICATION_PAGE_SIZE as usize);

    let mut search_samples = Vec::with_capacity(SEARCH_SAMPLE_COUNT);
    for _ in 0..SEARCH_SAMPLE_COUNT {
        let started = Instant::now();
        let search_page = repository
            .query_machines(&query(Some("qualification needle"), 0))
            .expect("search full catalog");
        search_samples.push(started.elapsed());
        assert_eq!(search_page.total, 1);
        assert_eq!(search_page.items.len(), 1);
        assert_eq!(
            search_page.items[0].short_name,
            format!("perf{:06}", QUALIFICATION_MACHINE_COUNT - 1)
        );
    }
    search_samples.sort_unstable();
    let search_median = search_samples[SEARCH_SAMPLE_COUNT / 2];
    let search_max = *search_samples.last().expect("search samples");

    let tail_started = Instant::now();
    let tail_page = repository
        .query_machines(&query(
            None,
            QUALIFICATION_MACHINE_COUNT - QUALIFICATION_PAGE_SIZE,
        ))
        .expect("load tail library page");
    let tail_elapsed = tail_started.elapsed();

    assert_eq!(tail_page.total, u64::from(QUALIFICATION_MACHINE_COUNT));
    assert_eq!(tail_page.items.len(), QUALIFICATION_PAGE_SIZE as usize);

    println!(
        "MT-410 library-performance catalog_rows={} page_size={} startup_ms={} search_median_ms={} search_max_ms={} tail_page_ms={}",
        QUALIFICATION_MACHINE_COUNT,
        QUALIFICATION_PAGE_SIZE,
        startup_elapsed.as_millis(),
        search_median.as_millis(),
        search_max.as_millis(),
        tail_elapsed.as_millis()
    );

    assert!(
        startup_elapsed <= STARTUP_QUERY_BUDGET,
        "first-page startup query took {:?}, budget is {:?}",
        startup_elapsed,
        STARTUP_QUERY_BUDGET
    );
    assert!(
        search_median <= SEARCH_QUERY_BUDGET,
        "median full-catalog search took {:?}, budget is {:?}",
        search_median,
        SEARCH_QUERY_BUDGET
    );
    assert!(
        tail_elapsed <= TAIL_PAGE_QUERY_BUDGET,
        "tail-page query took {:?}, budget is {:?}",
        tail_elapsed,
        TAIL_PAGE_QUERY_BUDGET
    );
}

fn query(text: Option<&str>, offset: u32) -> MachineQuery {
    MachineQuery {
        text: text.map(str::to_owned),
        manufacturer: None,
        year: None,
        driver_status: None,
        clone_filter: CloneFilter::All,
        sort: MachineSort::DescriptionAsc,
        include_devices: false,
        limit: QUALIFICATION_PAGE_SIZE,
        offset,
    }
}

fn seed_catalog(path: &Path, machine_count: u32) {
    let mut repository = CatalogRepository::open(path).expect("create qualification catalog");
    let transaction = repository
        .connection
        .transaction()
        .expect("start qualification seed transaction");

    transaction
        .execute(
            r#"INSERT INTO metadata_generation(
                source_kind, trust, executable_path, mame_version, mame_build,
                raw_version_line, listxml_build, mame_config, generated_at_epoch_ms,
                imported_at_epoch_ms, machine_count, active
            ) VALUES (
                'external', 'userConfigured', '/tmp/mt410-mame', 'mt410', 'synthetic',
                'mt410 synthetic', 'mt410 synthetic', '10', 1, 2, ?1, 1
            )"#,
            [i64::from(machine_count)],
        )
        .expect("seed active metadata generation");
    let generation_id = transaction.last_insert_rowid();

    {
        let mut insert = transaction
            .prepare(
                r#"INSERT INTO machines(
                    generation_id, short_name, description, year, manufacturer, source_file,
                    clone_of, is_bios, is_device, is_mechanical, runnable, driver_status
                ) VALUES (?1, ?2, ?3, ?4, ?5, 'mt410/perf.cpp', ?6, 0, 0, 0, 1, 'good')"#,
            )
            .expect("prepare qualification machine insert");

        for index in 0..machine_count {
            let short_name = format!("perf{index:06}");
            let description = if index + 1 == machine_count {
                format!("Qualification Needle {index:06}")
            } else {
                format!("Machine {index:06}")
            };
            let year = format!("{:04}", 1970 + (index % 55));
            let manufacturer = format!("Maker {:03}", index % 500);
            let clone_of = (index != 0 && index % 7 == 0).then_some("perf000000");

            insert
                .execute(params![
                    generation_id,
                    short_name,
                    description,
                    year,
                    manufacturer,
                    clone_of
                ])
                .expect("insert qualification machine");
        }
    }

    transaction
        .commit()
        .expect("commit qualification catalog seed");
}

struct TempCatalog {
    path: PathBuf,
}

impl TempCatalog {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        Self {
            path: std::env::temp_dir().join(format!(
                "mame-tauri-mt410-{}-{nonce}.sqlite3",
                std::process::id()
            )),
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempCatalog {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let candidate = PathBuf::from(format!("{}{}", self.path.display(), suffix));
            let _ = fs::remove_file(candidate);
        }
    }
}
