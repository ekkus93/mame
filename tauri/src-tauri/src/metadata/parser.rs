use std::{collections::HashMap, io::BufRead, str};

use quick_xml::{
    encoding::Decoder,
    escape::{resolve_predefined_entity, unescape},
    events::{BytesCData, BytesRef, BytesStart, BytesText, Event},
    Reader,
};

use crate::errors::{AppError, AppResult};

use super::model::{
    ChipMetadata, DeviceMetadata, DisplayMetadata, DriverMetadata, ListXmlSummary, MachineMetadata,
    SoftwareListAssociation,
};

pub(crate) fn parse_listxml<R, F>(input: R, mut on_machine: F) -> AppResult<ListXmlSummary>
where
    R: BufRead,
    F: FnMut(MachineMetadata) -> AppResult<()>,
{
    let mut reader = Reader::from_reader(input);
    reader.config_mut().trim_text(true);

    let mut buffer = Vec::new();
    let mut root_seen = false;
    let mut root_closed = false;
    let mut root_build = None;
    let mut mame_config = None;
    let mut current_machine: Option<MachineBuilder> = None;
    let mut text_target: Option<TextTarget> = None;
    let mut machine_count = 0_u64;

    loop {
        buffer.clear();
        let event = reader.read_event_into(&mut buffer).map_err(|error| {
            AppError::new(
                "MAME_METADATA_XML_INVALID",
                "MAME -listxml output is not valid XML.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;

        match event {
            Event::Start(start) => match start.name().as_ref() {
                b"mame" => {
                    if root_seen {
                        return Err(structure_error(
                            "MAME -listxml contains multiple root elements.",
                        ));
                    }
                    let attributes = attributes(&start, reader.decoder())?;
                    root_build = attributes.get("build").cloned();
                    mame_config = attributes.get("mameconfig").cloned();
                    root_seen = true;
                }
                b"machine" => {
                    if !root_seen || root_closed || current_machine.is_some() {
                        return Err(structure_error(
                            "MAME -listxml contains an invalid machine element position.",
                        ));
                    }
                    current_machine =
                        Some(MachineBuilder::new(attributes(&start, reader.decoder())?)?);
                }
                b"description" => {
                    set_text_target(&current_machine, &mut text_target, TextTarget::Description)?
                }
                b"year" => set_text_target(&current_machine, &mut text_target, TextTarget::Year)?,
                b"manufacturer" => {
                    set_text_target(&current_machine, &mut text_target, TextTarget::Manufacturer)?
                }
                b"device" => {
                    let machine = machine_mut(&mut current_machine)?;
                    if machine.current_device.is_some() {
                        return Err(structure_error(
                            "MAME -listxml contains a nested device element.",
                        ));
                    }
                    machine.current_device =
                        Some(parse_device(&attributes(&start, reader.decoder())?)?);
                }
                _ => {}
            },
            Event::Empty(empty) => {
                let name = empty.name();
                let attributes = attributes(&empty, reader.decoder())?;
                match name.as_ref() {
                    b"driver" => {
                        machine_mut(&mut current_machine)?.machine.driver =
                            Some(parse_driver(&attributes)?);
                    }
                    b"chip" => machine_mut(&mut current_machine)?
                        .machine
                        .chips
                        .push(parse_chip(&attributes)?),
                    b"display" => machine_mut(&mut current_machine)?
                        .machine
                        .displays
                        .push(parse_display(&attributes)?),
                    b"device" => machine_mut(&mut current_machine)?
                        .machine
                        .devices
                        .push(parse_device(&attributes)?),
                    b"instance" => {
                        let device = current_device_mut(&mut current_machine)?;
                        device.instance_name = Some(required_attr(&attributes, "name")?.to_owned());
                        device.instance_brief_name =
                            Some(required_attr(&attributes, "briefname")?.to_owned());
                    }
                    b"extension" => current_device_mut(&mut current_machine)?
                        .extensions
                        .push(required_attr(&attributes, "name")?.to_owned()),
                    b"softwarelist" => machine_mut(&mut current_machine)?
                        .machine
                        .software_lists
                        .push(parse_software_list(&attributes)?),
                    _ => {}
                }
            }
            Event::Text(text) => {
                if let Some(target) = text_target {
                    append_text(
                        machine_mut(&mut current_machine)?,
                        target,
                        decode_text(&text)?,
                    )?;
                }
            }
            Event::CData(cdata) => {
                if let Some(target) = text_target {
                    append_text(
                        machine_mut(&mut current_machine)?,
                        target,
                        decode_cdata(&cdata)?,
                    )?;
                }
            }
            Event::GeneralRef(reference) => {
                if let Some(target) = text_target {
                    append_text(
                        machine_mut(&mut current_machine)?,
                        target,
                        decode_reference(&reference)?,
                    )?;
                }
            }
            Event::End(end) => match end.name().as_ref() {
                b"description" if text_target == Some(TextTarget::Description) => {
                    text_target = None;
                }
                b"year" if text_target == Some(TextTarget::Year) => text_target = None,
                b"manufacturer" if text_target == Some(TextTarget::Manufacturer) => {
                    text_target = None;
                }
                b"device" => {
                    let machine = machine_mut(&mut current_machine)?;
                    let device = machine.current_device.take().ok_or_else(|| {
                        structure_error("MAME -listxml closed a device that was not open.")
                    })?;
                    machine.machine.devices.push(device);
                }
                b"machine" => {
                    if text_target.is_some() {
                        return Err(structure_error(
                            "MAME -listxml closed a machine while a text field was still open.",
                        ));
                    }
                    let builder = current_machine.take().ok_or_else(|| {
                        structure_error("MAME -listxml closed a machine that was not open.")
                    })?;
                    let machine = builder.finish()?;
                    on_machine(machine)?;
                    machine_count = machine_count.checked_add(1).ok_or_else(|| {
                        AppError::new(
                            "MAME_METADATA_MACHINE_COUNT_OVERFLOW",
                            "MAME metadata contains more machines than can be counted.",
                        )
                    })?;
                }
                b"mame" => {
                    if current_machine.is_some() {
                        return Err(structure_error(
                            "MAME -listxml closed its root while a machine was still open.",
                        ));
                    }
                    root_closed = true;
                }
                _ => {}
            },
            Event::Eof => break,
            Event::Decl(_) | Event::DocType(_) | Event::Comment(_) | Event::PI(_) => {}
        }
    }

    if !root_seen || !root_closed {
        return Err(structure_error(
            "MAME -listxml did not contain one complete mame root element.",
        ));
    }
    if current_machine.is_some() {
        return Err(structure_error(
            "MAME -listxml ended before the current machine was complete.",
        ));
    }
    if machine_count == 0 {
        return Err(AppError::new(
            "MAME_METADATA_EMPTY",
            "MAME -listxml did not contain any machine records.",
        ));
    }

    Ok(ListXmlSummary {
        build: root_build,
        mame_config,
        machine_count,
    })
}

#[derive(Debug)]
struct MachineBuilder {
    machine: MachineMetadata,
    current_device: Option<DeviceMetadata>,
}

impl MachineBuilder {
    fn new(attributes: HashMap<String, String>) -> AppResult<Self> {
        Ok(Self {
            machine: MachineMetadata {
                short_name: required_attr(&attributes, "name")?.to_owned(),
                description: String::new(),
                year: None,
                manufacturer: None,
                source_file: attributes.get("sourcefile").cloned(),
                clone_of: attributes.get("cloneof").cloned(),
                rom_of: attributes.get("romof").cloned(),
                is_bios: yes_no_attr(&attributes, "isbios", false)?,
                is_device: yes_no_attr(&attributes, "isdevice", false)?,
                is_mechanical: yes_no_attr(&attributes, "ismechanical", false)?,
                runnable: yes_no_attr(&attributes, "runnable", true)?,
                driver: None,
                chips: Vec::new(),
                displays: Vec::new(),
                devices: Vec::new(),
                software_lists: Vec::new(),
            },
            current_device: None,
        })
    }

    fn finish(self) -> AppResult<MachineMetadata> {
        if self.current_device.is_some() {
            return Err(structure_error(
                "MAME -listxml ended a machine before its device element was complete.",
            ));
        }
        if self.machine.description.trim().is_empty() {
            return Err(AppError::new(
                "MAME_METADATA_MACHINE_DESCRIPTION_MISSING",
                "A MAME machine record has no description.",
            )
            .with_details(serde_json::json!({ "machine": self.machine.short_name })));
        }
        Ok(self.machine)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextTarget {
    Description,
    Year,
    Manufacturer,
}

fn set_text_target(
    current_machine: &Option<MachineBuilder>,
    text_target: &mut Option<TextTarget>,
    next: TextTarget,
) -> AppResult<()> {
    if current_machine.is_none() || text_target.is_some() {
        return Err(structure_error(
            "MAME -listxml contains an invalid metadata text element position.",
        ));
    }
    *text_target = Some(next);
    Ok(())
}

fn append_text(machine: &mut MachineBuilder, target: TextTarget, value: String) -> AppResult<()> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(());
    }

    match target {
        TextTarget::Description => append_fragment(&mut machine.machine.description, value),
        TextTarget::Year => append_optional_fragment(&mut machine.machine.year, value),
        TextTarget::Manufacturer => {
            append_optional_fragment(&mut machine.machine.manufacturer, value)
        }
    }
    Ok(())
}

fn append_fragment(target: &mut String, value: &str) {
    if !target.is_empty() {
        target.push(' ');
    }
    target.push_str(value);
}

fn append_optional_fragment(target: &mut Option<String>, value: &str) {
    match target {
        Some(existing) => append_fragment(existing, value),
        None => *target = Some(value.to_owned()),
    }
}

fn machine_mut(current_machine: &mut Option<MachineBuilder>) -> AppResult<&mut MachineBuilder> {
    current_machine.as_mut().ok_or_else(|| {
        structure_error("MAME -listxml contains machine metadata outside a machine element.")
    })
}

fn current_device_mut(
    current_machine: &mut Option<MachineBuilder>,
) -> AppResult<&mut DeviceMetadata> {
    machine_mut(current_machine)?
        .current_device
        .as_mut()
        .ok_or_else(|| {
            structure_error("MAME -listxml contains device details outside a device element.")
        })
}

fn parse_driver(attributes: &HashMap<String, String>) -> AppResult<DriverMetadata> {
    Ok(DriverMetadata {
        status: required_attr(attributes, "status")?.to_owned(),
        emulation: required_attr(attributes, "emulation")?.to_owned(),
        cocktail: attributes.get("cocktail").cloned(),
        savestate: required_attr(attributes, "savestate")?.to_owned(),
        requires_artwork: yes_no_attr(attributes, "requiresartwork", false)?,
        unofficial: yes_no_attr(attributes, "unofficial", false)?,
        no_sound_hardware: yes_no_attr(attributes, "nosoundhardware", false)?,
        incomplete: yes_no_attr(attributes, "incomplete", false)?,
    })
}

fn parse_chip(attributes: &HashMap<String, String>) -> AppResult<ChipMetadata> {
    Ok(ChipMetadata {
        chip_type: required_attr(attributes, "type")?.to_owned(),
        name: required_attr(attributes, "name")?.to_owned(),
        tag: attributes.get("tag").cloned(),
        clock_hz: parse_optional_u64(attributes, "clock")?,
    })
}

fn parse_display(attributes: &HashMap<String, String>) -> AppResult<DisplayMetadata> {
    let rotate = parse_optional_u16(attributes, "rotate")?;
    if let Some(rotate) = rotate {
        if !matches!(rotate, 0 | 90 | 180 | 270) {
            return Err(invalid_attribute("display", "rotate", rotate.to_string()));
        }
    }

    let refresh = required_attr(attributes, "refresh")?;
    let refresh_hz = refresh
        .parse::<f64>()
        .map_err(|_| invalid_attribute("display", "refresh", refresh.to_owned()))?;
    if !refresh_hz.is_finite() || refresh_hz <= 0.0 {
        return Err(invalid_attribute("display", "refresh", refresh.to_owned()));
    }

    Ok(DisplayMetadata {
        tag: attributes.get("tag").cloned(),
        display_type: required_attr(attributes, "type")?.to_owned(),
        rotate,
        flip_x: yes_no_attr(attributes, "flipx", false)?,
        width: parse_optional_u32(attributes, "width")?,
        height: parse_optional_u32(attributes, "height")?,
        refresh_hz,
        pixel_clock_hz: parse_optional_u64(attributes, "pixclock")?,
    })
}

fn parse_device(attributes: &HashMap<String, String>) -> AppResult<DeviceMetadata> {
    Ok(DeviceMetadata {
        device_type: required_attr(attributes, "type")?.to_owned(),
        tag: attributes.get("tag").cloned(),
        fixed_image: attributes.get("fixed_image").cloned(),
        mandatory: attributes.get("mandatory").cloned(),
        interface: attributes.get("interface").cloned(),
        instance_name: None,
        instance_brief_name: None,
        extensions: Vec::new(),
    })
}

fn parse_software_list(attributes: &HashMap<String, String>) -> AppResult<SoftwareListAssociation> {
    Ok(SoftwareListAssociation {
        tag: required_attr(attributes, "tag")?.to_owned(),
        name: required_attr(attributes, "name")?.to_owned(),
        status: required_attr(attributes, "status")?.to_owned(),
        filter: attributes.get("filter").cloned(),
    })
}

fn attributes(start: &BytesStart<'_>, decoder: Decoder) -> AppResult<HashMap<String, String>> {
    let mut result = HashMap::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(|error| {
            AppError::new(
                "MAME_METADATA_XML_ATTRIBUTE_INVALID",
                "MAME -listxml contains a malformed XML attribute.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        let key = str::from_utf8(attribute.key.as_ref()).map_err(|error| {
            AppError::new(
                "MAME_METADATA_XML_ATTRIBUTE_INVALID",
                "MAME -listxml contains a non-UTF-8 XML attribute name.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        let value = attribute
            .decode_and_unescape_value(decoder)
            .map_err(|error| {
                AppError::new(
                    "MAME_METADATA_XML_ATTRIBUTE_INVALID",
                    "MAME -listxml contains an XML attribute value that cannot be decoded.",
                )
                .with_details(serde_json::json!({ "cause": error.to_string() }))
            })?
            .into_owned();
        if result.insert(key.to_owned(), value).is_some() {
            return Err(AppError::new(
                "MAME_METADATA_XML_ATTRIBUTE_DUPLICATE",
                "MAME -listxml contains a duplicate XML attribute.",
            )
            .with_details(serde_json::json!({ "attribute": key })));
        }
    }
    Ok(result)
}

fn decode_text(text: &BytesText<'_>) -> AppResult<String> {
    let decoded = text.decode().map_err(|error| {
        AppError::new(
            "MAME_METADATA_XML_TEXT_INVALID",
            "MAME -listxml contains text that cannot be decoded.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    unescape(&decoded)
        .map(|value| value.into_owned())
        .map_err(|error| {
            AppError::new(
                "MAME_METADATA_XML_TEXT_INVALID",
                "MAME -listxml contains invalid XML text escaping.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })
}

fn decode_reference(reference: &BytesRef<'_>) -> AppResult<String> {
    if let Some(character) = reference.resolve_char_ref().map_err(|error| {
        AppError::new(
            "MAME_METADATA_XML_TEXT_INVALID",
            "MAME -listxml contains an invalid XML character reference.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })? {
        return Ok(character.to_string());
    }

    let name = str::from_utf8(reference.as_ref()).map_err(|error| {
        AppError::new(
            "MAME_METADATA_XML_TEXT_INVALID",
            "MAME -listxml contains an XML entity reference that cannot be decoded.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    resolve_predefined_entity(name)
        .map(str::to_owned)
        .ok_or_else(|| {
            AppError::new(
                "MAME_METADATA_XML_ENTITY_UNSUPPORTED",
                "MAME -listxml contains an unsupported XML entity reference.",
            )
            .with_details(serde_json::json!({ "entity": name }))
        })
}

fn decode_cdata(cdata: &BytesCData<'_>) -> AppResult<String> {
    cdata
        .decode()
        .map(|value| value.into_owned())
        .map_err(|error| {
            AppError::new(
                "MAME_METADATA_XML_TEXT_INVALID",
                "MAME -listxml contains CDATA that cannot be decoded.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })
}

fn required_attr<'a>(attributes: &'a HashMap<String, String>, name: &str) -> AppResult<&'a str> {
    attributes
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::new(
                "MAME_METADATA_REQUIRED_ATTRIBUTE_MISSING",
                "MAME -listxml is missing a required metadata attribute.",
            )
            .with_details(serde_json::json!({ "attribute": name }))
        })
}

fn yes_no_attr(attributes: &HashMap<String, String>, name: &str, default: bool) -> AppResult<bool> {
    match attributes.get(name).map(String::as_str) {
        None => Ok(default),
        Some("yes") => Ok(true),
        Some("no") => Ok(false),
        Some(value) => Err(invalid_attribute("boolean", name, value.to_owned())),
    }
}

fn parse_optional_u16(attributes: &HashMap<String, String>, name: &str) -> AppResult<Option<u16>> {
    parse_optional_number(attributes, name)
}

fn parse_optional_u32(attributes: &HashMap<String, String>, name: &str) -> AppResult<Option<u32>> {
    parse_optional_number(attributes, name)
}

fn parse_optional_u64(attributes: &HashMap<String, String>, name: &str) -> AppResult<Option<u64>> {
    parse_optional_number(attributes, name)
}

fn parse_optional_number<T>(
    attributes: &HashMap<String, String>,
    name: &str,
) -> AppResult<Option<T>>
where
    T: str::FromStr,
{
    let Some(value) = attributes.get(name) else {
        return Ok(None);
    };
    value
        .parse::<T>()
        .map(Some)
        .map_err(|_| invalid_attribute("numeric", name, value.clone()))
}

fn invalid_attribute(element: &str, attribute: &str, value: String) -> AppError {
    AppError::new(
        "MAME_METADATA_ATTRIBUTE_INVALID",
        "MAME -listxml contains an invalid metadata attribute value.",
    )
    .with_details(serde_json::json!({
        "element": element,
        "attribute": attribute,
        "value": value
    }))
}

fn structure_error(message: &str) -> AppError {
    AppError::new("MAME_METADATA_XML_STRUCTURE_INVALID", message)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::parse_listxml;

    const FIXTURE: &str = include_str!("../../tests/fixtures/listxml-representative.xml");

    #[test]
    fn parses_representative_machine_metadata() {
        let mut machines = Vec::new();
        let summary = parse_listxml(Cursor::new(FIXTURE.as_bytes()), |machine| {
            machines.push(machine);
            Ok(())
        })
        .expect("representative fixture must parse");

        assert_eq!(summary.build.as_deref(), Some("0.288 test-fixture"));
        assert_eq!(summary.mame_config.as_deref(), Some("10"));
        assert_eq!(summary.machine_count, 4);

        let parent = machines
            .iter()
            .find(|machine| machine.short_name == "galaxian")
            .expect("galaxian fixture machine");
        assert_eq!(parent.description, "Galaxian (Namco set 1)");
        assert_eq!(parent.manufacturer.as_deref(), Some("Namco & Co."));
        assert_eq!(
            parent.driver.as_ref().map(|driver| driver.status.as_str()),
            Some("good")
        );
        assert_eq!(parent.displays.len(), 1);
        assert_eq!(parent.displays[0].width, Some(768));
        assert_eq!(parent.displays[0].height, Some(224));
        assert_eq!(parent.chips.len(), 2);

        let apple = machines
            .iter()
            .find(|machine| machine.short_name == "apple2e")
            .expect("apple2e fixture machine");
        assert_eq!(apple.software_lists.len(), 2);
        assert_eq!(apple.devices.len(), 1);
        assert_eq!(apple.devices[0].extensions, vec!["dsk", "do", "po"]);

        let clone = machines
            .iter()
            .find(|machine| machine.short_name == "galaxiana")
            .expect("clone fixture machine");
        assert_eq!(clone.clone_of.as_deref(), Some("galaxian"));

        let device = machines
            .iter()
            .find(|machine| machine.short_name == "z80")
            .expect("device fixture machine");
        assert!(device.is_device);
        assert!(!device.runnable);
    }

    #[test]
    fn unknown_elements_are_ignored_without_losing_known_fields() {
        let xml = r#"<?xml version="1.0"?>
<mame build="test" mameconfig="10">
  <machine name="future" sourcefile="future.cpp">
    <description>Future Machine</description>
    <future-container><future-child answer="42"/></future-container>
    <year>2026</year>
    <manufacturer>Example</manufacturer>
    <driver status="good" emulation="good" savestate="supported"/>
  </machine>
</mame>"#;
        let mut machines = Vec::new();
        parse_listxml(Cursor::new(xml.as_bytes()), |machine| {
            machines.push(machine);
            Ok(())
        })
        .expect("unknown elements must be forward-compatible");

        assert_eq!(machines[0].year.as_deref(), Some("2026"));
        assert_eq!(machines[0].manufacturer.as_deref(), Some("Example"));
    }

    #[test]
    fn unicode_text_round_trips_without_loss() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<mame build="unicode-test" mameconfig="10">
  <machine name="unicode" sourcefile="unicode.cpp">
    <description>日本語テスト – Café</description>
    <year>2026</year>
    <manufacturer>株式会社テスト &amp; Société</manufacturer>
    <driver status="good" emulation="good" savestate="supported"/>
  </machine>
</mame>"#;
        let mut machines = Vec::new();
        parse_listxml(Cursor::new(xml.as_bytes()), |machine| {
            machines.push(machine);
            Ok(())
        })
        .expect("Unicode metadata must parse");

        assert_eq!(machines[0].description, "日本語テスト – Café");
        assert_eq!(
            machines[0].manufacturer.as_deref(),
            Some("株式会社テスト & Société")
        );
    }

    #[test]
    fn malformed_xml_is_rejected() {
        let xml = b"<mame><machine name='bad'><description>Broken</mame>";
        let error = parse_listxml(Cursor::new(xml.as_slice()), |_| Ok(()))
            .expect_err("malformed XML must fail");
        assert!(matches!(
            error.code.as_str(),
            "MAME_METADATA_XML_INVALID" | "MAME_METADATA_XML_STRUCTURE_INVALID"
        ));
    }

    #[test]
    fn empty_catalog_is_rejected_explicitly() {
        let xml = b"<mame build='test' mameconfig='10'></mame>";
        let error = parse_listxml(Cursor::new(xml.as_slice()), |_| Ok(()))
            .expect_err("empty catalog must fail");
        assert_eq!(error.code, "MAME_METADATA_EMPTY");
    }
}
