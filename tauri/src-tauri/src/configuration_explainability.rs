use serde::Serialize;

use crate::{
    config::{AudioPreference, LaunchPreferencesV1, RendererPreference, WindowPreference},
    configuration_precedence::{
        resolve_effective_value, ConfigurationAssignment, ConfigurationLayer,
        EffectiveConfigurationValue,
    },
    errors::{AppError, AppResult},
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PreferenceExplanation<T> {
    pub effective_value: T,
    pub source_layer: ConfigurationLayer,
    pub pending_launch_override: Option<T>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPreferencesExplanation {
    pub window_mode: PreferenceExplanation<WindowPreference>,
    pub renderer: PreferenceExplanation<RendererPreference>,
    pub audio: PreferenceExplanation<AudioPreference>,
}

impl LaunchPreferencesExplanation {
    pub fn effective_preferences(&self) -> LaunchPreferencesV1 {
        LaunchPreferencesV1 {
            window_mode: self.window_mode.effective_value,
            renderer: self.renderer.effective_value,
            audio: self.audio.effective_value,
        }
    }
}

/// Explain the app-owned launch-preference layers using the MT-602 precedence contract.
///
/// `Inherit` at the MAME/global layer is a deliberate sentinel meaning the app will
/// emit no command-line override for that option. It does not claim knowledge of the
/// concrete provider/value MAME ultimately selects from its native configuration stack.
pub fn explain_launch_preferences(
    general: &LaunchPreferencesV1,
    machine: &LaunchPreferencesV1,
    pending: Option<&LaunchPreferencesV1>,
) -> AppResult<LaunchPreferencesExplanation> {
    Ok(LaunchPreferencesExplanation {
        window_mode: explain_preference(
            WindowPreference::Inherit,
            general.window_mode,
            machine.window_mode,
            pending.map(|value| value.window_mode),
        )?,
        renderer: explain_preference(
            RendererPreference::Inherit,
            general.renderer,
            machine.renderer,
            pending.map(|value| value.renderer),
        )?,
        audio: explain_preference(
            AudioPreference::Inherit,
            general.audio,
            machine.audio,
            pending.map(|value| value.audio),
        )?,
    })
}

pub fn resolve_effective_launch_preferences(
    general: &LaunchPreferencesV1,
    machine: &LaunchPreferencesV1,
    pending: Option<&LaunchPreferencesV1>,
) -> AppResult<LaunchPreferencesV1> {
    Ok(explain_launch_preferences(general, machine, pending)?.effective_preferences())
}

fn explain_preference<T>(
    inherit: T,
    general: T,
    machine: T,
    pending: Option<T>,
) -> AppResult<PreferenceExplanation<T>>
where
    T: Copy + PartialEq,
{
    let mut assignments = vec![ConfigurationAssignment::new(
        ConfigurationLayer::MameGlobalDefaults,
        inherit,
    )];

    if general != inherit {
        assignments.push(ConfigurationAssignment::new(
            ConfigurationLayer::ProfileDefaults,
            general,
        ));
    }
    if machine != inherit {
        assignments.push(ConfigurationAssignment::new(
            ConfigurationLayer::MachineOverrides,
            machine,
        ));
    }

    let pending_launch_override = pending.filter(|value| *value != inherit);
    if let Some(value) = pending_launch_override {
        assignments.push(ConfigurationAssignment::new(
            ConfigurationLayer::TransientLaunchOverrides,
            value,
        ));
    }

    let EffectiveConfigurationValue {
        value: effective_value,
        source_layer,
    } = resolve_effective_value(assignments)?.ok_or_else(|| {
        AppError::new(
            "CONFIG_EXPLANATION_EMPTY",
            "Configuration explainability unexpectedly produced no effective value.",
        )
    })?;

    Ok(PreferenceExplanation {
        effective_value,
        source_layer,
        pending_launch_override,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{AudioPreference, LaunchPreferencesV1, RendererPreference, WindowPreference},
        configuration_precedence::ConfigurationLayer,
    };

    use super::{explain_launch_preferences, resolve_effective_launch_preferences};

    fn general() -> LaunchPreferencesV1 {
        LaunchPreferencesV1 {
            window_mode: WindowPreference::Windowed,
            renderer: RendererPreference::Bgfx,
            audio: AudioPreference::Auto,
        }
    }

    #[test]
    fn machine_override_explains_effective_value_and_source_layer() {
        let machine = LaunchPreferencesV1 {
            window_mode: WindowPreference::Fullscreen,
            renderer: RendererPreference::Inherit,
            audio: AudioPreference::Disabled,
        };

        let explanation = explain_launch_preferences(&general(), &machine, None)
            .expect("valid layered preferences must explain");
        assert_eq!(
            explanation.window_mode.effective_value,
            WindowPreference::Fullscreen
        );
        assert_eq!(
            explanation.window_mode.source_layer,
            ConfigurationLayer::MachineOverrides
        );
        assert_eq!(
            explanation.renderer.effective_value,
            RendererPreference::Bgfx
        );
        assert_eq!(
            explanation.renderer.source_layer,
            ConfigurationLayer::ProfileDefaults
        );
    }

    #[test]
    fn transient_override_wins_and_is_explicitly_reported_as_pending() {
        let machine = LaunchPreferencesV1 {
            window_mode: WindowPreference::Fullscreen,
            renderer: RendererPreference::Inherit,
            audio: AudioPreference::Inherit,
        };
        let pending = LaunchPreferencesV1 {
            window_mode: WindowPreference::Windowed,
            renderer: RendererPreference::Software,
            audio: AudioPreference::Inherit,
        };

        let explanation = explain_launch_preferences(&general(), &machine, Some(&pending))
            .expect("pending preferences must explain");
        assert_eq!(
            explanation.window_mode.effective_value,
            WindowPreference::Windowed
        );
        assert_eq!(
            explanation.window_mode.source_layer,
            ConfigurationLayer::TransientLaunchOverrides
        );
        assert_eq!(
            explanation.window_mode.pending_launch_override,
            Some(WindowPreference::Windowed)
        );
        assert_eq!(
            explanation.renderer.pending_launch_override,
            Some(RendererPreference::Software)
        );
        assert_eq!(explanation.audio.pending_launch_override, None);
    }

    #[test]
    fn all_inherited_values_explain_that_mame_global_configuration_owns_the_value() {
        let inherited = LaunchPreferencesV1::default();
        let explanation = explain_launch_preferences(&inherited, &inherited, None)
            .expect("inherited preferences must explain");

        assert_eq!(
            explanation.renderer.source_layer,
            ConfigurationLayer::MameGlobalDefaults
        );
        assert_eq!(
            explanation.renderer.effective_value,
            RendererPreference::Inherit
        );
        assert_eq!(explanation.renderer.pending_launch_override, None);
    }

    #[test]
    fn effective_resolution_matches_the_explanation_winners() {
        let machine = LaunchPreferencesV1 {
            window_mode: WindowPreference::Fullscreen,
            renderer: RendererPreference::Inherit,
            audio: AudioPreference::Disabled,
        };
        let pending = LaunchPreferencesV1 {
            window_mode: WindowPreference::Inherit,
            renderer: RendererPreference::Software,
            audio: AudioPreference::Inherit,
        };

        assert_eq!(
            resolve_effective_launch_preferences(&general(), &machine, Some(&pending))
                .expect("effective preferences"),
            LaunchPreferencesV1 {
                window_mode: WindowPreference::Fullscreen,
                renderer: RendererPreference::Software,
                audio: AudioPreference::Disabled,
            }
        );
    }
}
