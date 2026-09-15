use std::{collections::HashMap, io::BufRead, str};

use quick_xml::{
    escape::{resolve_predefined_entity, unescape},
    events::{BytesCData, BytesRef, BytesStart, BytesText, Event},
    Reader, XmlVersion,
};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum SoftwareListFilter {
    #[default]
    All,
    Parents,
    Clones,
    Year,
    Publisher,
    Supported,
    PartiallySupported,
    Unsupported,
}

impl SoftwareListFilter {
    pub(crate) fn requires_value(self) -> bool {
        matches!(self, Self::Year | Self::Publisher)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SoftwarePartSummary {
    pub name: String,
    pub interface: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareItemSummary {
    pub short_name: String,
    pub description: String,
    pub year: String,
    pub publisher: String,
    pub clone_of: Option<String>,
    pub supported: String,
    pub parts: Vec<SoftwarePartSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedSoftwareList {
    pub name: String,
    pub description: Option<String>,
    pub total: u64,
    pub items: Vec<SoftwareItemSummary>,
}

pub(crate) fn parse_software_list_page<R: BufRead>(
    input: R,
    expected_list: &str,
    text_filter: Option<&str>,
    filter: SoftwareListFilter,
    filter_value: Option<&str>,
    limit: u32,
    offset: u32,
) -> AppResult<ParsedSoftwareList> {
    parse_software_list(
        input,
        expected_list,
        text_filter,
        filter,
        filter_value,
        limit,
        offset,
        None,
    )
}

pub(crate) fn parse_software_item<R: BufRead>(
    input: R,
    expected_list: &str,
    exact_short_name: &str,
) -> AppResult<Option<SoftwareItemSummary>> {
    let parsed = parse_software_list(
        input,
        expected_list,
        None,
        SoftwareListFilter::All,
        None,
        1,
        0,
        Some(exact_short_name),
    )?;
    Ok(parsed.items.into_iter().next())
}

#[allow(clippy::too_many_arguments)]
fn parse_software_list<R: BufRead>(
    input: R,
    expected_list: &str,
    text_filter: Option<&str>,
    filter: SoftwareListFilter,
    filter_value: Option<&str>,
    limit: u32,
    offset: u32,
    exact_short_name: Option<&str>,
) -> AppResult<ParsedSoftwareList> {
    if filter.requires_value() && filter_value.is_none() {
        return Err(AppError::new(
            "MAME_SOFTWARE_FILTER_VALUE_REQUIRED",
            "The selected software filter requires a value.",
        ));
    }

    let mut reader = Reader::from_reader(input);
    reader.config_mut().trim_text(true);

    let text_filter = text_filter.map(|value| value.to_lowercase());
    let normalized_filter_value = filter_value
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let page_start = u64::from(offset);
    let page_end = page_start.saturating_add(u64::from(limit));
    let mut buffer = Vec::new();
    let mut root_seen = false;
    let mut root_closed = false;
    let mut list_seen = false;
    let mut list_closed = false;
    let mut list_description = None;
    let mut current_item: Option<SoftwareItemBuilder> = None;
    let mut text_target = None;
    let mut total = 0_u64;
    let mut items = Vec::new();

    loop {
        buffer.clear();
        let event = reader.read_event_into(&mut buffer).map_err(xml_error)?;
        match event {
            Event::Start(start) => match start.name().as_ref() {
                b"softwarelists" => {
                    if root_seen {
                        return Err(structure_error(
                            "MAME -getsoftlist returned multiple root elements.",
                        ));
                    }
                    root_seen = true;
                }
                b"softwarelist" => {
                    if !root_seen || root_closed || list_seen || current_item.is_some() {
                        return Err(structure_error(
                            "MAME -getsoftlist returned an invalid softwarelist element position.",
                        ));
                    }
                    let attributes = attributes(&start)?;
                    let name = required_attr(&attributes, "name")?;
                    if name != expected_list {
                        return Err(AppError::new(
                            "MAME_SOFTWARE_LIST_MISMATCH",
                            "MAME returned a software list different from the requested list.",
                        )
                        .with_details(serde_json::json!({
                            "requested": expected_list,
                            "returned": name
                        })));
                    }
                    list_description = attributes.get("description").cloned();
                    list_seen = true;
                }
                b"software" => {
                    if !list_seen || list_closed || current_item.is_some() {
                        return Err(structure_error(
                            "MAME -getsoftlist returned an invalid software item position.",
                        ));
                    }
                    current_item = Some(SoftwareItemBuilder::new(attributes(&start)?)?);
                }
                b"description" => {
                    set_text_target(&current_item, &mut text_target, TextTarget::Description)?
                }
                b"year" => set_text_target(&current_item, &mut text_target, TextTarget::Year)?,
                b"publisher" => {
                    set_text_target(&current_item, &mut text_target, TextTarget::Publisher)?
                }
                b"part" => {
                    item_mut(&mut current_item)?
                        .parts
                        .push(parse_part(&attributes(&start)?)?);
                }
                _ => {}
            },
            Event::Empty(empty) => {
                if empty.name().as_ref() == b"part" {
                    item_mut(&mut current_item)?
                        .parts
                        .push(parse_part(&attributes(&empty)?)?);
                }
            }
            Event::Text(text) => {
                if let Some(target) = text_target {
                    append_text(item_mut(&mut current_item)?, target, decode_text(&text)?);
                }
            }
            Event::CData(cdata) => {
                if let Some(target) = text_target {
                    append_text(item_mut(&mut current_item)?, target, decode_cdata(&cdata)?);
                }
            }
            Event::GeneralRef(reference) => {
                if let Some(target) = text_target {
                    append_text(
                        item_mut(&mut current_item)?,
                        target,
                        decode_reference(&reference)?,
                    );
                }
            }
            Event::End(end) => match end.name().as_ref() {
                b"description" if text_target == Some(TextTarget::Description) => {
                    text_target = None;
                }
                b"year" if text_target == Some(TextTarget::Year) => text_target = None,
                b"publisher" if text_target == Some(TextTarget::Publisher) => text_target = None,
                b"software" => {
                    if text_target.is_some() {
                        return Err(structure_error(
                            "MAME -getsoftlist closed a software item while metadata text was open.",
                        ));
                    }
                    let item = current_item
                        .take()
                        .ok_or_else(|| {
                            structure_error("MAME closed a software item that was not open.")
                        })?
                        .finish()?;
                    let exact_matches = exact_short_name
                        .map(|name| item.short_name == name)
                        .unwrap_or(true);
                    if exact_matches
                        && item_matches(
                            &item,
                            text_filter.as_deref(),
                            filter,
                            normalized_filter_value,
                        )
                    {
                        if total >= page_start && total < page_end {
                            items.push(item);
                        }
                        total = total.checked_add(1).ok_or_else(|| {
                            AppError::new(
                                "MAME_SOFTWARE_RESULT_COUNT_OVERFLOW",
                                "The software list contains more matching items than can be counted.",
                            )
                        })?;
                    }
                }
                b"softwarelist" => {
                    if current_item.is_some() {
                        return Err(structure_error(
                            "MAME -getsoftlist closed the list while a software item was open.",
                        ));
                    }
                    list_closed = true;
                }
                b"softwarelists" => {
                    if current_item.is_some() || (list_seen && !list_closed) {
                        return Err(structure_error(
                            "MAME -getsoftlist closed its root before the list was complete.",
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

    if !root_seen || !root_closed || !list_seen || !list_closed {
        return Err(structure_error(
            "MAME -getsoftlist did not return one complete software list document.",
        ));
    }

    Ok(ParsedSoftwareList {
        name: expected_list.to_owned(),
        description: list_description,
        total,
        items,
    })
}

#[derive(Debug)]
struct SoftwareItemBuilder {
    short_name: String,
    description: String,
    year: String,
    publisher: String,
    clone_of: Option<String>,
    supported: String,
    parts: Vec<SoftwarePartSummary>,
}

impl SoftwareItemBuilder {
    fn new(attributes: HashMap<String, String>) -> AppResult<Self> {
        let supported = attributes
            .get("supported")
            .map(String::as_str)
            .unwrap_or("yes");
        if !matches!(supported, "yes" | "partial" | "no") {
            return Err(AppError::new(
                "MAME_SOFTWARE_ATTRIBUTE_INVALID",
                "MAME -getsoftlist returned an invalid support status.",
            )
            .with_details(serde_json::json!({ "supported": supported })));
        }
        Ok(Self {
            short_name: required_attr(&attributes, "name")?.to_owned(),
            description: String::new(),
            year: String::new(),
            publisher: String::new(),
            clone_of: attributes.get("cloneof").cloned(),
            supported: supported.to_owned(),
            parts: Vec::new(),
        })
    }

    fn finish(self) -> AppResult<SoftwareItemSummary> {
        if self.description.trim().is_empty()
            || self.year.trim().is_empty()
            || self.publisher.trim().is_empty()
        {
            return Err(AppError::new(
                "MAME_SOFTWARE_METADATA_INCOMPLETE",
                "A MAME software item is missing required descriptive metadata.",
            )
            .with_details(serde_json::json!({ "shortName": self.short_name })));
        }
        Ok(SoftwareItemSummary {
            short_name: self.short_name,
            description: self.description,
            year: self.year,
            publisher: self.publisher,
            clone_of: self.clone_of,
            supported: self.supported,
            parts: self.parts,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextTarget {
    Description,
    Year,
    Publisher,
}

fn set_text_target(
    current_item: &Option<SoftwareItemBuilder>,
    text_target: &mut Option<TextTarget>,
    target: TextTarget,
) -> AppResult<()> {
    if current_item.is_none() || text_target.is_some() {
        return Err(structure_error(
            "MAME -getsoftlist returned descriptive metadata outside a software item.",
        ));
    }
    *text_target = Some(target);
    Ok(())
}

fn item_mut(current_item: &mut Option<SoftwareItemBuilder>) -> AppResult<&mut SoftwareItemBuilder> {
    current_item.as_mut().ok_or_else(|| {
        structure_error("MAME -getsoftlist returned software metadata outside a software item.")
    })
}

fn append_text(item: &mut SoftwareItemBuilder, target: TextTarget, value: String) {
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    let destination = match target {
        TextTarget::Description => &mut item.description,
        TextTarget::Year => &mut item.year,
        TextTarget::Publisher => &mut item.publisher,
    };
    if !destination.is_empty() {
        destination.push(' ');
    }
    destination.push_str(value);
}

fn parse_part(attributes: &HashMap<String, String>) -> AppResult<SoftwarePartSummary> {
    Ok(SoftwarePartSummary {
        name: required_attr(attributes, "name")?.to_owned(),
        interface: required_attr(attributes, "interface")?.to_owned(),
    })
}

fn item_matches(
    item: &SoftwareItemSummary,
    text_filter: Option<&str>,
    filter: SoftwareListFilter,
    filter_value: Option<&str>,
) -> bool {
    let text_matches = match text_filter {
        None => true,
        Some(text_filter) => [
            item.short_name.as_str(),
            item.description.as_str(),
            item.year.as_str(),
            item.publisher.as_str(),
        ]
        .iter()
        .any(|value| value.to_lowercase().contains(text_filter)),
    };
    if !text_matches {
        return false;
    }

    match filter {
        SoftwareListFilter::All => true,
        SoftwareListFilter::Parents => item.clone_of.is_none(),
        SoftwareListFilter::Clones => item.clone_of.is_some(),
        SoftwareListFilter::Year => filter_value.is_some_and(|value| item.year == value),
        SoftwareListFilter::Publisher => {
            filter_value.is_some_and(|value| item.publisher.eq_ignore_ascii_case(value))
        }
        SoftwareListFilter::Supported => item.supported == "yes",
        SoftwareListFilter::PartiallySupported => item.supported == "partial",
        SoftwareListFilter::Unsupported => item.supported == "no",
    }
}

fn attributes(start: &BytesStart<'_>) -> AppResult<HashMap<String, String>> {
    let mut result = HashMap::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(|error| {
            AppError::new(
                "MAME_SOFTWARE_XML_ATTRIBUTE_INVALID",
                "MAME -getsoftlist contains a malformed XML attribute.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        let key = str::from_utf8(attribute.key.as_ref()).map_err(|error| {
            AppError::new(
                "MAME_SOFTWARE_XML_ATTRIBUTE_INVALID",
                "MAME -getsoftlist contains a non-UTF-8 XML attribute name.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        let value = attribute
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(|error| {
                AppError::new(
                    "MAME_SOFTWARE_XML_ATTRIBUTE_INVALID",
                    "MAME -getsoftlist contains an XML attribute value that cannot be decoded.",
                )
                .with_details(serde_json::json!({ "cause": error.to_string() }))
            })?
            .into_owned();
        if result.insert(key.to_owned(), value).is_some() {
            return Err(AppError::new(
                "MAME_SOFTWARE_XML_ATTRIBUTE_DUPLICATE",
                "MAME -getsoftlist contains a duplicate XML attribute.",
            )
            .with_details(serde_json::json!({ "attribute": key })));
        }
    }
    Ok(result)
}

fn required_attr<'a>(attributes: &'a HashMap<String, String>, name: &str) -> AppResult<&'a str> {
    attributes
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::new(
                "MAME_SOFTWARE_REQUIRED_ATTRIBUTE_MISSING",
                "MAME -getsoftlist is missing a required attribute.",
            )
            .with_details(serde_json::json!({ "attribute": name }))
        })
}

fn decode_text(text: &BytesText<'_>) -> AppResult<String> {
    let decoded = text.decode().map_err(text_error)?;
    unescape(&decoded)
        .map(|value| value.into_owned())
        .map_err(text_error)
}

fn decode_cdata(cdata: &BytesCData<'_>) -> AppResult<String> {
    cdata
        .decode()
        .map(|value| value.into_owned())
        .map_err(text_error)
}

fn decode_reference(reference: &BytesRef<'_>) -> AppResult<String> {
    if let Some(character) = reference.resolve_char_ref().map_err(text_error)? {
        return Ok(character.to_string());
    }
    let name = str::from_utf8(reference.as_ref()).map_err(text_error)?;
    resolve_predefined_entity(name)
        .map(str::to_owned)
        .ok_or_else(|| {
            AppError::new(
                "MAME_SOFTWARE_XML_ENTITY_UNSUPPORTED",
                "MAME -getsoftlist contains an unsupported XML entity reference.",
            )
            .with_details(serde_json::json!({ "entity": name }))
        })
}

fn xml_error(error: quick_xml::Error) -> AppError {
    AppError::new(
        "MAME_SOFTWARE_XML_INVALID",
        "MAME -getsoftlist output is not valid XML.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

fn text_error(error: impl std::fmt::Display) -> AppError {
    AppError::new(
        "MAME_SOFTWARE_XML_TEXT_INVALID",
        "MAME -getsoftlist contains text that cannot be decoded.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

fn structure_error(message: &str) -> AppError {
    AppError::new("MAME_SOFTWARE_XML_STRUCTURE_INVALID", message)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{parse_software_item, parse_software_list_page, SoftwareListFilter};

    const FIXTURE: &str = r#"<?xml version="1.0"?>
<softwarelists>
  <softwarelist name="apple2_flop_orig" description="Apple II original disks">
    <software name="agentusa">
      <description>Agent U.S.A.</description>
      <year>1984</year>
      <publisher>Scholastic</publisher>
      <part name="flop1" interface="floppy_5_25"><dataarea name="flop" size="1"/></part>
    </software>
    <software name="airheart" supported="partial">
      <description>Airheart</description>
      <year>1986</year>
      <publisher>Broderbund</publisher>
      <part name="flop1" interface="floppy_5_25"/>
      <part name="flop2" interface="floppy_5_25"/>
    </software>
    <software name="archon2" cloneof="archon" supported="no">
      <description>Archon II: Adept</description>
      <year>1984</year>
      <publisher>Electronic Arts</publisher>
    </software>
    <software name="archon">
      <description>Archon</description>
      <year>1983</year>
      <publisher>Free Fall Associates</publisher>
      <part name="flop1" interface="floppy_5_25"/>
    </software>
  </softwarelist>
</softwarelists>"#;

    #[test]
    fn parses_filters_pages_and_parts() {
        let parsed = parse_software_list_page(
            Cursor::new(FIXTURE.as_bytes()),
            "apple2_flop_orig",
            Some("1984"),
            SoftwareListFilter::All,
            None,
            1,
            1,
        )
        .expect("software fixture must parse");
        assert_eq!(
            parsed.description.as_deref(),
            Some("Apple II original disks")
        );
        assert_eq!(parsed.total, 2);
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].short_name, "archon2");
        assert_eq!(parsed.items[0].clone_of.as_deref(), Some("archon"));
    }

    #[test]
    fn applies_exact_mame_software_filters_before_pagination() {
        let parsed = parse_software_list_page(
            Cursor::new(FIXTURE.as_bytes()),
            "apple2_flop_orig",
            None,
            SoftwareListFilter::PartiallySupported,
            None,
            20,
            0,
        )
        .expect("support filter");
        assert_eq!(parsed.total, 1);
        assert_eq!(parsed.items[0].short_name, "airheart");
        assert_eq!(parsed.items[0].parts.len(), 2);

        let publisher = parse_software_list_page(
            Cursor::new(FIXTURE.as_bytes()),
            "apple2_flop_orig",
            None,
            SoftwareListFilter::Publisher,
            Some("electronic arts"),
            20,
            0,
        )
        .expect("publisher filter");
        assert_eq!(publisher.total, 1);
        assert_eq!(publisher.items[0].short_name, "archon2");
    }

    #[test]
    fn exact_item_lookup_is_not_displaced_by_an_earlier_fuzzy_match() {
        let fuzzy = parse_software_list_page(
            Cursor::new(FIXTURE.as_bytes()),
            "apple2_flop_orig",
            Some("archon"),
            SoftwareListFilter::All,
            None,
            1,
            0,
        )
        .expect("fuzzy search");
        assert_eq!(fuzzy.items[0].short_name, "archon2");

        let exact = parse_software_item(
            Cursor::new(FIXTURE.as_bytes()),
            "apple2_flop_orig",
            "archon",
        )
        .expect("exact item lookup")
        .expect("exact item must exist");
        assert_eq!(exact.short_name, "archon");
        assert_eq!(exact.parts.len(), 1);
    }

    #[test]
    fn rejects_missing_filter_values() {
        let error = parse_software_list_page(
            Cursor::new(FIXTURE.as_bytes()),
            "apple2_flop_orig",
            None,
            SoftwareListFilter::Year,
            None,
            20,
            0,
        )
        .expect_err("value filter must fail closed");
        assert_eq!(error.code, "MAME_SOFTWARE_FILTER_VALUE_REQUIRED");
    }

    #[test]
    fn rejects_a_different_list_than_requested() {
        let error = parse_software_list_page(
            Cursor::new(FIXTURE.as_bytes()),
            "different_list",
            None,
            SoftwareListFilter::All,
            None,
            50,
            0,
        )
        .expect_err("mismatched list must fail");
        assert_eq!(error.code, "MAME_SOFTWARE_LIST_MISMATCH");
    }
}
