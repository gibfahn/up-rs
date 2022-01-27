use std::collections::HashMap;

use assert_cmd::cargo::cargo_bin;
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
}"#;

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
    envs.insert("up_binary_path", cargo_bin("up"));
    cmd.envs(envs);

    cmd.args(
        [
            "--config",
            temp_dir.join("up_config_dir/up.yaml").to_str().unwrap(),
        ]
        .iter(),
    );
    cmd.assert().success();

    // Link Task: Check symlinks were created correctly.
    assert::link(
        &temp_dir.join("link_dir/home_dir/file_to_link"),
        &temp_dir.join("link_dir/dotfile_dir/file_to_link"),
    );

    #[cfg(target_os = "macos")]
    {
        use duct::cmd;

        // Defaults Task: Check values were set correctly.
        let actual_value = cmd!("defaults", "read", test_plist).read().unwrap();
        assert_eq!(actual_value, EXPECTED_DEFAULTS_VALUE);

        // Defaults Task: Check types were set correctly.

        assert_eq!(
            "Type is boolean",
            cmd!(
                "defaults",
                "read-type",
                "co.fahn.up-rs.test-up_run_passing",
                "NSNavPanelExpandedStateForSaveMode"
            )
            .read()
            .unwrap()
        );

        assert_eq!(
            "Type is float",
            cmd!(
                "defaults",
                "read-type",
                "co.fahn.up-rs.test-up_run_passing",
                "autohide-time-modifier"
            )
            .read()
            .unwrap()
        );

        assert_eq!(
            "Type is integer",
            cmd!(
                "defaults",
                "read-type",
                "co.fahn.up-rs.test-up_run_passing",
                "AppleKeyboardUIMode"
            )
            .read()
            .unwrap()
        );

        assert_eq!(
            "Type is array",
            cmd!(
                "defaults",
                "read-type",
                "co.fahn.up-rs.test-up_run_passing",
                "CustomHeaders"
            )
            .read()
            .unwrap()
        );

        assert_eq!(
            "Type is dictionary",
            cmd!(
                "defaults",
                "read-type",
                "co.fahn.up-rs.test-up_run_passing",
                "AppleICUDateFormatStrings"
            )
            .read()
            .unwrap()
        );
    }
}
