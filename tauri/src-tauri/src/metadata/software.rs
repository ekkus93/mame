use std::{collections::HashMap, io::BufRead, str};

use quick_xml::{
    escape::{resolve_predefined_entity, unescape},
    events::{BytesCData, BytesRef, BytesStart, BytesText, Event},
    Reader, XmlVersion,
};
use serde::Serialize;

use crate::errors::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareItemSummary {
    pub short_name: String,
    pub description: String,
    pub year: String,
    pub publisher: String,
    pub clone_of: Option<String>,
    pub supported: String,
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
    limit: u32,
    offset: u32,
) -> AppResult<ParsedSoftwareList> {
    let mut reader = Reader::from_reader(input);
    reader.config_mut().trim_text(true);

    let filter = text_filter.map(|value| value.to_lowercase());
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
                _ => {}
            },
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
                    if item_matches(&item, filter.as_deref()) {
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
            Event::Empty(_)
            | Event::Decl(_)
            | Event::DocType(_)
            | Event::Comment(_)
            | Event::PI(_) => {}
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

fn item_matches(item: &SoftwareItemSummary, filter: Option<&str>) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    [
        item.short_name.as_str(),
        item.description.as_str(),
        item.year.as_str(),
        item.publisher.as_str(),
    ]
    .iter()
    .any(|value| value.to_lowercase().contains(filter))
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

    use super::parse_software_list_page;

    const FIXTURE: &str = r#"<?xml version="1.0"?>
<softwarelists>
  <softwarelist name="apple2_flop_orig" description="Apple II original disks">
    <software name="agentusa">
      <description>Agent U.S.A.</description>
      <year>1984</year>
      <publisher>Scholastic</publisher>
    </software>
    <software name="airheart" supported="partial">
      <description>Airheart</description>
      <year>1986</year>
      <publisher>Broderbund</publisher>
    </software>
    <software name="archon2" cloneof="archon">
      <description>Archon II: Adept</description>
      <year>1984</year>
      <publisher>Electronic Arts</publisher>
    </software>
  </softwarelist>
</softwarelists>"#;

    #[test]
    fn parses_filters_and_pages_known_software_metadata() {
        let parsed = parse_software_list_page(
            Cursor::new(FIXTURE.as_bytes()),
            "apple2_flop_orig",
            Some("1984"),
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
    fn rejects_a_different_list_than_requested() {
        let error = parse_software_list_page(
            Cursor::new(FIXTURE.as_bytes()),
            "different_list",
            None,
            50,
            0,
        )
        .expect_err("mismatched list must fail");
        assert_eq!(error.code, "MAME_SOFTWARE_LIST_MISMATCH");
    }
}
