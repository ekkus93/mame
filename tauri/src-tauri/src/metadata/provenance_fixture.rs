use std::io::Cursor;

use serde_json::Value;

use super::parser::parse_listxml;

const CAPTURED_FIXTURE: &str =
    include_str!("../../../tests/fixtures/listxml-representative-captured.xml");
const PROVENANCE: &str =
    include_str!("../../../tests/fixtures/listxml-representative.provenance.json");

#[test]
fn captured_fixture_matches_recorded_mame_identity_and_representative_shape() {
    let provenance: Value =
        serde_json::from_str(PROVENANCE).expect("valid fixture provenance JSON");
    assert_eq!(provenance["schemaVersion"], 1);
    assert_eq!(provenance["mameVersionLine"], "0.264 (unknown)");
    assert_eq!(provenance["packageVersion"], "0.264+dfsg.1-1");
    assert_eq!(provenance["githubActionsRunId"], 34688122578_u64);

    let mut machines = Vec::new();
    let summary = parse_listxml(Cursor::new(CAPTURED_FIXTURE.as_bytes()), |machine| {
        machines.push(machine);
        Ok(())
    })
    .expect("captured MAME fixture must parse");

    assert_eq!(summary.build.as_deref(), Some("0.264 (unknown)"));
    assert_eq!(summary.mame_config.as_deref(), Some("10"));
    assert_eq!(summary.machine_count, 4);

    let observed: Vec<_> = machines
        .iter()
        .map(|machine| machine.short_name.as_str())
        .collect();
    assert_eq!(observed, ["galaxian", "galaxiana", "apple2e", "z80"]);

    let clone = machines
        .iter()
        .find(|machine| machine.short_name == "galaxiana")
        .expect("captured clone");
    assert_eq!(clone.clone_of.as_deref(), Some("galaxian"));

    let apple = machines
        .iter()
        .find(|machine| machine.short_name == "apple2e")
        .expect("captured software-list machine");
    assert_eq!(apple.software_lists.len(), 3);
    assert_eq!(apple.devices.len(), 3);

    let device = machines
        .iter()
        .find(|machine| machine.short_name == "z80")
        .expect("captured device record");
    assert!(device.is_device);
    assert!(!device.runnable);
}
