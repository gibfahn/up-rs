use std::collections::HashMap;

use testutils::assert;

#[cfg(target_os = "macos")]
const EXPECTED_DEFAULTS_VALUE: &str = r#"{
    AppleICUDateFormatStrings =     {
        1 = "y-MM-dd";
        2 = "y-MM-dd";
        3 = "y-MM-dd";
        4 = "EEEE, y-MM-dd";
    };
    AppleKeyboardUIMode = 2;
    AppleWindowTabbingMode = always;
    CustomHeaders =     (
        "List-ID",
        "Message-ID",
        "X-Member-Count"
    );
    HintCharacters = "tnseriaodhplfuwyq;gjvmc,x.z/bk4738291056";
    MJConfigFile = "~/.config/hammerspoon/init.lua";
    NSNavPanelExpandedStateForSaveMode = 1;
    NSNavPanelExpandedStateForSaveMode2 = 0;
    "_FXShowPosixPathInTitle" = 1;
    "autohide-time-modifier" = "0.25";
}
"#;

/// Run a full up with a bunch of configuration and check things work.
#[test]
fn up_run_passing() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    testutils::copy_all(
        &testutils::fixture_dir(testutils::function_path!()),
        &temp_dir,
    )
    .unwrap();

    #[cfg(target_os = "macos")]
    let test_plist = "co.fahn.up-rs.test-up_run_passing";

    #[cfg(target_os = "macos")]
    {
        let mut cmd = std::process::Command::new("defaults");
        cmd.args(&["delete", test_plist]);
        cmd.output().unwrap();
    }

    let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
    let mut envs = HashMap::new();
    // Used in link task.
    envs.insert("link_from_dir", temp_dir.join("link_dir/dotfile_dir"));
    envs.insert("link_to_dir", temp_dir.join("link_dir/home_dir"));
    envs.insert("up_binary_path", testutils::test_binary_path("up"));
    cmd.envs(envs);

    cmd.args(
        [
            "--config",
            temp_dir.join("up_config_dir/up.yaml").to_str().unwrap(),
        ]
        .iter(),
    );
    let cmd_output = testutils::run_cmd(&mut cmd);
    assert!(
        cmd_output.status.success(),
        "\n Up command should pass successfully.",
    );

    // Link Task: Check symlinks were created correctly.
    assert::link(
        &temp_dir.join("link_dir/home_dir/file_to_link"),
        &temp_dir.join("link_dir/dotfile_dir/file_to_link"),
    );

    #[cfg(target_os = "macos")]
    {
        use testutils::run_defaults;

        // Defaults Task: Check values were set correctly.
        let actual_value = run_defaults(&["read", test_plist]);
        assert_eq!(actual_value, EXPECTED_DEFAULTS_VALUE);

        // Defaults Task: Check types were set correctly.

        assert_eq!(
            "Type is boolean\n",
            run_defaults(&[
                "read-type",
                "co.fahn.up-rs.test-up_run_passing",
                "NSNavPanelExpandedStateForSaveMode"
            ])
        );

        assert_eq!(
            "Type is float\n",
            run_defaults(&[
                "read-type",
                "co.fahn.up-rs.test-up_run_passing",
                "autohide-time-modifier"
            ])
        );

        assert_eq!(
            "Type is integer\n",
            run_defaults(&[
                "read-type",
                "co.fahn.up-rs.test-up_run_passing",
                "AppleKeyboardUIMode"
            ])
        );

        assert_eq!(
            "Type is array\n",
            run_defaults(&[
                "read-type",
                "co.fahn.up-rs.test-up_run_passing",
                "CustomHeaders"
            ])
        );

        assert_eq!(
            "Type is dictionary\n",
            run_defaults(&[
                "read-type",
                "co.fahn.up-rs.test-up_run_passing",
                "AppleICUDateFormatStrings"
            ])
        );
    }
}
