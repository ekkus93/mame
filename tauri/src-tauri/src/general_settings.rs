        let saved = apply_launch_preferences(&path, preferences.clone())
            .expect("persist launch preferences");
        assert_eq!(saved.launch_preferences, preferences);
        assert_eq!(
            load_general_settings(&path)
                .expect("reload general settings")
                .launch_preferences,
            preferences
        );

        fs::remove_dir_all(root).expect("remove settings root");
    }

    #[cfg(unix)]
    #[test]
    fn external_override_round_trip_and_reset_preserve_other_settings() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_root("runtime-override");
        fs::create_dir_all(&root).expect("create settings root");
        let path = root.join("settings.json");
        let executable = root.join("custom mame");
        fs::write(
            &executable,
            "#!/bin/sh\nif [ \"$1\" = '-noreadconfig' ] && [ \"$2\" = '-version' ]; then echo '0.288 override-test'; exit 0; fi\nexit 1\n",
        )
        .expect("write fake MAME");
        let mut permissions = fs::metadata(&executable).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).expect("mark executable");

        let preferences = LaunchPreferencesV1 {
            window_mode: WindowPreference::Windowed,
            renderer: RendererPreference::Bgfx,
            audio: AudioPreference::Disabled,
        };
        apply_launch_preferences(&path, preferences.clone()).expect("seed launch preferences");

        let overridden =
            apply_mame_executable(&path, Some(executable.to_string_lossy().into_owned()))
                .expect("validated external override");
        assert_eq!(overridden.mame_executable.as_deref(), executable.to_str());
        assert_eq!(overridden.launch_preferences, preferences);

        let bundled = apply_mame_executable(&path, None).expect("reset to bundled runtime");
        assert_eq!(bundled.mame_executable, None);
        assert_eq!(bundled.launch_preferences, preferences);
        assert_eq!(
            load_general_settings(&path)
                .expect("reload reset settings")
                .mame_executable,
            None
        );

        fs::remove_dir_all(root).expect("remove settings root");
    }

    #[test]
    fn invalid_external_override_does_not_replace_bundled_default() {
        let root = temp_root("invalid-runtime-override");
        fs::create_dir_all(&root).expect("create settings root");
        let path = root.join("settings.json");

        let error = apply_mame_executable(
            &path,
            Some(root.join("missing-mame").to_string_lossy().into_owned()),
        )
        .expect_err("invalid override must fail validation");
        assert_eq!(error.code, "MAME_EXECUTABLE_NOT_FOUND");
        assert_eq!(
            load_general_settings(&path)
                .expect("bundled default remains")
                .mame_executable,
            None
        );
