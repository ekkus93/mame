//! Deterministic application-level configuration precedence.
//!
//! This module defines the layers the Tauri frontend owns around MAME's native
//! configuration machinery. MAME's compiled defaults and standard INI stack
//! are represented as one supplied `MameGlobalDefaults` layer here; MAME still
//! owns the internal ordering of those native sources.

use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};

/// Application-owned precedence layers, ordered from lowest to highest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConfigurationLayer {
    ApplicationDefaults,
    MameGlobalDefaults,
    ProfileDefaults,
    MachineOverrides,
    TransientLaunchOverrides,
}

impl ConfigurationLayer {
    pub const LOW_TO_HIGH: [Self; 5] = [
        Self::ApplicationDefaults,
        Self::MameGlobalDefaults,
        Self::ProfileDefaults,
        Self::MachineOverrides,
        Self::TransientLaunchOverrides,
    ];

    pub const HIGH_TO_LOW: [Self; 5] = [
        Self::TransientLaunchOverrides,
        Self::MachineOverrides,
        Self::ProfileDefaults,
        Self::MameGlobalDefaults,
        Self::ApplicationDefaults,
    ];
}

/// One value supplied by exactly one application precedence layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationAssignment<T> {
    pub layer: ConfigurationLayer,
    pub value: T,
}

impl<T> ConfigurationAssignment<T> {
    pub fn new(layer: ConfigurationLayer, value: T) -> Self {
        Self { layer, value }
    }
}

/// The requested value after applying application-level precedence.
///
/// `source_layer` explains why the value won. It does not claim that MAME's
/// runtime backend ultimately accepted the requested value; renderer/audio
/// providers may still fall back and must be reported separately when runtime
/// observation is available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveConfigurationValue<T> {
    pub value: T,
    pub source_layer: ConfigurationLayer,
}

/// Resolve one option using the fixed MT-602 precedence contract.
///
/// Cross-layer conflicts are resolved by the higher layer. Multiple values for
/// the same layer are rejected instead of relying on insertion order, because
/// same-layer last-write-wins behavior would make effective values depend on
/// collection order rather than the documented configuration model.
pub fn resolve_effective_value<T>(
    assignments: impl IntoIterator<Item = ConfigurationAssignment<T>>,
) -> AppResult<Option<EffectiveConfigurationValue<T>>> {
    let mut application_defaults = None;
    let mut mame_global_defaults = None;
    let mut profile_defaults = None;
    let mut machine_overrides = None;
    let mut transient_launch_overrides = None;

    for assignment in assignments {
        let slot = match assignment.layer {
            ConfigurationLayer::ApplicationDefaults => &mut application_defaults,
            ConfigurationLayer::MameGlobalDefaults => &mut mame_global_defaults,
            ConfigurationLayer::ProfileDefaults => &mut profile_defaults,
            ConfigurationLayer::MachineOverrides => &mut machine_overrides,
            ConfigurationLayer::TransientLaunchOverrides => &mut transient_launch_overrides,
        };

        if slot.is_some() {
            return Err(AppError::new(
                "CONFIG_PRECEDENCE_CONFLICT",
                "More than one value was supplied for the same configuration layer.",
            )
            .with_details(serde_json::json!({
                "layer": assignment.layer,
            })));
        }
        *slot = Some(assignment.value);
    }

    let winner = if let Some(value) = transient_launch_overrides {
        Some((ConfigurationLayer::TransientLaunchOverrides, value))
    } else if let Some(value) = machine_overrides {
        Some((ConfigurationLayer::MachineOverrides, value))
    } else if let Some(value) = profile_defaults {
        Some((ConfigurationLayer::ProfileDefaults, value))
    } else if let Some(value) = mame_global_defaults {
        Some((ConfigurationLayer::MameGlobalDefaults, value))
    } else {
        application_defaults.map(|value| (ConfigurationLayer::ApplicationDefaults, value))
    };

    Ok(
        winner.map(|(source_layer, value)| EffectiveConfigurationValue {
            value,
            source_layer,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_effective_value, ConfigurationAssignment, ConfigurationLayer,
        EffectiveConfigurationValue,
    };

    fn assignment(layer: ConfigurationLayer, value: &str) -> ConfigurationAssignment<String> {
        ConfigurationAssignment::new(layer, value.to_owned())
    }

    #[test]
    fn precedence_order_is_explicit_and_stable() {
        assert_eq!(
            ConfigurationLayer::LOW_TO_HIGH,
            [
                ConfigurationLayer::ApplicationDefaults,
                ConfigurationLayer::MameGlobalDefaults,
                ConfigurationLayer::ProfileDefaults,
                ConfigurationLayer::MachineOverrides,
                ConfigurationLayer::TransientLaunchOverrides,
            ]
        );
        assert_eq!(
            ConfigurationLayer::HIGH_TO_LOW,
            [
                ConfigurationLayer::TransientLaunchOverrides,
                ConfigurationLayer::MachineOverrides,
                ConfigurationLayer::ProfileDefaults,
                ConfigurationLayer::MameGlobalDefaults,
                ConfigurationLayer::ApplicationDefaults,
            ]
        );
    }

    #[test]
    fn highest_present_layer_wins_regardless_of_input_order() {
        let resolved = resolve_effective_value([
            assignment(ConfigurationLayer::MachineOverrides, "machine"),
            assignment(ConfigurationLayer::ApplicationDefaults, "application"),
            assignment(ConfigurationLayer::TransientLaunchOverrides, "transient"),
            assignment(ConfigurationLayer::MameGlobalDefaults, "mame"),
            assignment(ConfigurationLayer::ProfileDefaults, "profile"),
        ])
        .expect("valid layered values must resolve");

        assert_eq!(
            resolved,
            Some(EffectiveConfigurationValue {
                value: "transient".to_owned(),
                source_layer: ConfigurationLayer::TransientLaunchOverrides,
            })
        );
    }

    #[test]
    fn removing_higher_layers_reveals_the_next_value() {
        let resolved = resolve_effective_value([
            assignment(ConfigurationLayer::ApplicationDefaults, "application"),
            assignment(ConfigurationLayer::MameGlobalDefaults, "mame"),
            assignment(ConfigurationLayer::ProfileDefaults, "profile"),
            assignment(ConfigurationLayer::MachineOverrides, "machine"),
        ])
        .expect("valid layered values must resolve");

        assert_eq!(
            resolved,
            Some(EffectiveConfigurationValue {
                value: "machine".to_owned(),
                source_layer: ConfigurationLayer::MachineOverrides,
            })
        );
    }

    #[test]
    fn mame_global_defaults_override_application_defaults() {
        let resolved = resolve_effective_value([
            assignment(ConfigurationLayer::ApplicationDefaults, "application"),
            assignment(ConfigurationLayer::MameGlobalDefaults, "mame"),
        ])
        .expect("valid layered values must resolve");

        assert_eq!(
            resolved,
            Some(EffectiveConfigurationValue {
                value: "mame".to_owned(),
                source_layer: ConfigurationLayer::MameGlobalDefaults,
            })
        );
    }

    #[test]
    fn same_layer_duplicates_are_rejected_instead_of_becoming_order_dependent() {
        let error = resolve_effective_value([
            assignment(ConfigurationLayer::ProfileDefaults, "first"),
            assignment(ConfigurationLayer::ProfileDefaults, "second"),
        ])
        .expect_err("duplicate values in one layer must be rejected");

        assert_eq!(error.code, "CONFIG_PRECEDENCE_CONFLICT");
        assert_eq!(error.details["layer"], "profileDefaults");
    }

    #[test]
    fn an_unset_option_has_no_effective_value() {
        let resolved = resolve_effective_value::<String>([])
            .expect("an option absent from every layer is valid");
        assert_eq!(resolved, None);
    }
}
