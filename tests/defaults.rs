use std::process::Command;

use testutils::assert;

#[test]
fn defaults_read_global() {
    let temp_dir = testutils::temp_dir(file!(), testutils::function_name!()).unwrap();

    let expected_value = Command::new("defaults")
        .args(&["read", "-g", "com.apple.sound.beep.sound"])
        .output()
        .unwrap()
        .stdout;

    // Reading a normal value should have the same output as the defaults command (but yaml not
    // defaults own format).
    {
        let mut cmd = testutils::up_cmd(&temp_dir);
        cmd.args(&["defaults", "read", "-g", "com.apple.sound.beep.sound"]);
        let cmd_output = testutils::run_cmd(&mut cmd);
        assert!(cmd_output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&expected_value),
            String::from_utf8_lossy(&cmd_output.stdout)
        );
    }

    // Providing a full absolute path to a plist file should also work.
    {
        let mut cmd = testutils::up_cmd(&temp_dir);
        cmd.args(&[
            "defaults",
            "read",
            &format!(
                "{}/Library/Preferences/.GlobalPreferences.plist",
                dirs::home_dir().unwrap().display()
            ),
            "com.apple.sound.beep.sound",
        ]);
        let cmd_output = testutils::run_cmd(&mut cmd);
        assert!(cmd_output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&expected_value),
            String::from_utf8_lossy(&cmd_output.stdout)
        );
    }

    // Setting -g is the same as setting the domain NSGlobalDomain, so shouldn't pass both a key and
    // a value to `defaults read`.
    {
        let mut cmd = testutils::up_cmd(&temp_dir);
        cmd.args(&[
            "defaults",
            "read",
            "-g",
            "NSGlobalDomain",
            "com.apple.sound.beep.sound",
        ]);
        let cmd_output = testutils::run_cmd(&mut cmd);
        assert!(!cmd_output.status.success());
        assert::contains(
            &String::from_utf8_lossy(&cmd_output.stderr),
            "both a domain and a key",
        );
    }
}

#[test]
fn defaults_read_local() {
    let temp_dir = testutils::temp_dir(file!(), testutils::function_name!()).unwrap();

    // Four-letter codes for view modes: `icnv`, `clmv`, `glyv`, `Nlsv`
    let expected_value = Command::new("defaults")
        .args(&["read", "com.apple.finder", "FXPreferredViewStyle"])
        .output()
        .unwrap()
        .stdout;

    // Reading a normal value should have the same output as the defaults command (but yaml not
    // defaults own format).
    {
        let mut cmd = testutils::up_cmd(&temp_dir);
        cmd.args(&[
            "defaults",
            "read",
            "com.apple.finder",
            "FXPreferredViewStyle",
        ]);
        let cmd_output = testutils::run_cmd(&mut cmd);
        assert!(cmd_output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&expected_value),
            String::from_utf8_lossy(&cmd_output.stdout)
        );
    }

    // Providing a full absolute path to a plist file should also work.
    {
        let mut cmd = testutils::up_cmd(&temp_dir);
        cmd.args(&[
            "defaults",
            "read",
            &format!(
                "{}/Library/Preferences/com.apple.finder.plist",
                dirs::home_dir().unwrap().display()
            ),
            "FXPreferredViewStyle",
        ]);
        let cmd_output = testutils::run_cmd(&mut cmd);
        assert!(cmd_output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&expected_value),
            String::from_utf8_lossy(&cmd_output.stdout)
        );
    }
}
