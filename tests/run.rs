use std::{collections::HashMap, process::Command};

use testutils::{assert, run_defaults};

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
    let temp_dir = testutils::temp_dir(file!(), testutils::function_name!()).unwrap();

    testutils::copy_all(
        &testutils::fixtures_dir()
            .join(testutils::test_path(file!()))
            .join(testutils::function_name!()),
        &temp_dir,
    )
    .unwrap();

    let test_plist = format!("co.fahn.up-rs.test-{}", testutils::function_name!());
    {
        let mut cmd = Command::new("defaults");
        cmd.args(&["delete", &test_plist]);
        cmd.output().unwrap();
    }

    let mut cmd = testutils::up_cmd(&temp_dir);
    let mut envs = HashMap::new();
    // Used in link task.
    envs.insert("link_from_dir", temp_dir.join("link_dir/dotfile_dir"));
    envs.insert("link_to_dir", temp_dir.join("link_dir/home_dir"));
    envs.insert("up_binary_path", testutils::up_binary_path());
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

    // Defaults Task: Check values were set correctly.
    let actual_value = run_defaults(&["read", &test_plist]);
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
