use std::collections::BTreeMap;

use serde::Serialize;

use crate::mame::MameExecutableIdentity;

#[derive(Debug, Clone, PartialEq)]
pub struct MachineMetadata {
    pub short_name: String,
    pub description: String,
    pub year: Option<String>,
    pub manufacturer: Option<String>,
    pub source_file: Option<String>,
    pub clone_of: Option<String>,
    pub rom_of: Option<String>,
    pub is_bios: bool,
    pub is_device: bool,
    pub is_mechanical: bool,
    pub runnable: bool,
    pub driver: Option<DriverMetadata>,
    pub chips: Vec<ChipMetadata>,
    pub displays: Vec<DisplayMetadata>,
    pub devices: Vec<DeviceMetadata>,
    pub software_lists: Vec<SoftwareListAssociation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverMetadata {
    pub status: String,
    pub emulation: String,
    pub cocktail: Option<String>,
    pub savestate: String,
    pub requires_artwork: bool,
    pub unofficial: bool,
    pub no_sound_hardware: bool,
    pub incomplete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChipMetadata {
    pub chip_type: String,
    pub name: String,
    pub tag: Option<String>,
    pub clock_hz: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayMetadata {
    pub tag: Option<String>,
    pub display_type: String,
    pub rotate: Option<u16>,
    pub flip_x: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub refresh_hz: f64,
    pub pixel_clock_hz: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceMetadata {
    pub device_type: String,
    pub tag: Option<String>,
    pub fixed_image: Option<String>,
    pub mandatory: Option<String>,
    pub interface: Option<String>,
    pub instance_name: Option<String>,
    pub instance_brief_name: Option<String>,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftwareListAssociation {
    pub tag: String,
    pub name: String,
    pub status: String,
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachineSoftwareListInfo {
    pub tag: String,
    pub name: String,
    pub status: String,
    pub filter: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListXmlSummary {
    pub build: Option<String>,
    pub mame_config: Option<String>,
    pub machine_count: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataGenerationSummary {
    pub generation_id: i64,
    pub source_kind: String,
    pub trust: String,
    pub executable_path: String,
    pub mame_version: String,
    pub mame_build: Option<String>,
    pub raw_version_line: String,
    pub listxml_build: Option<String>,
    pub mame_config: Option<String>,
    pub generated_at_epoch_ms: u64,
    pub imported_at_epoch_ms: u64,
    pub machine_count: u64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MetadataFreshness {
    Empty,
    Fresh,
    Stale,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataStatus {
    pub schema_version: u32,
    pub freshness: MetadataFreshness,
    pub current_executable: MameExecutableIdentity,
    pub active_generation: Option<MetadataGenerationSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataRefreshResult {
    pub schema_version: u32,
    pub generation: MetadataGenerationSummary,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachineListItem {
    pub short_name: String,
    pub description: String,
    pub year: Option<String>,
    pub manufacturer: Option<String>,
    pub source_file: Option<String>,
    pub clone_of: Option<String>,
    pub runnable: bool,
    pub is_device: bool,
    pub driver_status: Option<String>,
    pub display_count: u32,
    pub software_list_count: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MachineDisplayInfo {
    pub tag: Option<String>,
    pub display_type: String,
    pub rotate: Option<i64>,
    pub flip_x: bool,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub refresh_hz: f64,
    pub pixel_clock_hz: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MachineDetail {
    pub schema_version: u32,
    pub generation_id: i64,
    pub short_name: String,
    pub description: String,
    pub year: Option<String>,
    pub manufacturer: Option<String>,
    pub source_file: Option<String>,
    pub clone_of: Option<String>,
    pub parent_description: Option<String>,
    pub rom_of: Option<String>,
    pub is_bios: bool,
    pub is_device: bool,
    pub is_mechanical: bool,
    pub runnable: bool,
    pub driver_status: Option<String>,
    pub driver_emulation: Option<String>,
    pub driver_cocktail: Option<String>,
    pub driver_savestate: Option<String>,
    pub driver_requires_artwork: bool,
    pub driver_unofficial: bool,
    pub driver_no_sound_hardware: bool,
    pub driver_incomplete: bool,
    pub displays: Vec<MachineDisplayInfo>,
    pub software_lists: Vec<MachineSoftwareListInfo>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MachineAvailability {
    Available,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachinePage {
    pub schema_version: u32,
    pub generation_id: i64,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
    pub items: Vec<MachineListItem>,
    pub availability_by_short_name: BTreeMap<String, MachineAvailability>,
}
