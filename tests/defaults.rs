// Defaults tests are macOS only.
#![cfg(target_os = "macos")]

use testutils::{assert, run_defaults};

#[test]
fn defaults_read_global() {
    let temp_dir = testutils::temp_dir(file!(), testutils::function_name!()).unwrap();

    let expected_value = run_defaults(&["read", "-g", "com.apple.sound.beep.sound"]);

    // Reading a normal value should have the same output as the defaults command (but yaml not
    // defaults own format).
    {
        let mut cmd = testutils::up_cmd(&temp_dir);
        cmd.args(&["defaults", "read", "-g", "com.apple.sound.beep.sound"]);
        let cmd_output = testutils::run_cmd(&mut cmd);
        assert!(cmd_output.status.success());
        assert_eq!(expected_value, String::from_utf8_lossy(&cmd_output.stdout));
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
        assert_eq!(expected_value, String::from_utf8_lossy(&cmd_output.stdout));
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
    let expected_value = run_defaults(&["read", "com.apple.finder", "FXPreferredViewStyle"]);

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
        assert_eq!(expected_value, String::from_utf8_lossy(&cmd_output.stdout));
    }

    // A .plist extension should be allowed too.
    {
        let mut cmd = testutils::up_cmd(&temp_dir);
        cmd.args(&[
            "defaults",
            "read",
            "com.apple.finder.plist",
            "FXPreferredViewStyle",
        ]);
        let cmd_output = testutils::run_cmd(&mut cmd);
        assert!(cmd_output.status.success());
        assert_eq!(expected_value, String::from_utf8_lossy(&cmd_output.stdout));
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
        assert_eq!(expected_value, String::from_utf8_lossy(&cmd_output.stdout));
    }
}

#[test]
fn defaults_write_local() {
    let temp_dir = testutils::temp_dir(file!(), testutils::function_name!()).unwrap();

    let domain = format!("co.fahn.up-rs.test-{}", testutils::function_name!());

    // Format: (defaults_type, orig_value, orig_check_value, new_value, check_value)
    let test_values = [
        ("bool", "true", "true", "false", "0"),
        ("int", "5", "5", "10", "10"),
        ("float", "5.123", "5.123000144958496", "7.8", "7.8"),
        (
            "string",
            "initial value\nline 2",
            "\"initial value\\nline 2\"",
            "\"new value\\nnew line 2\"",
            "new value\nnew line 2",
        ),
        (
            "array",
            "a b c",
            "- a\n- b\n- c",
            // Check swapping an array to a dict works.
            "x: four\ny: five\nz: six",
            "{\n    x = four;\n    y = five;\n    z = six;\n}",
        ),
        (
            "dict",
            "u one v two w three",
            "u: one\nv: two\nw: three",
            // Check swapping a dict to an array works.
            "[\"d\",\"e\",\"f\"]",
            "(\n    d,\n    e,\n    f\n)",
        ),
    ];

    for (n, (defaults_type, orig_value, _, _, _)) in test_values.iter().enumerate() {
        let values = match *defaults_type {
            "array" | "dict" => orig_value.split_whitespace().collect(),
            _ => vec![*orig_value],
        };

        let defaults_key = format!("defaults_write_local_{}", n);
        let defaults_type = format!("-{}", defaults_type);
        let mut args = vec!["write", &domain, &defaults_key, &defaults_type];
        args.extend(values);

        // Write the original value to a test plist file.
        run_defaults(&args);
    }

    // Check we agree with `defaults` about the original value.
    for (n, (_, _, orig_check_value, _, _)) in test_values.iter().enumerate() {
        let mut cmd = testutils::up_cmd(&temp_dir);
        cmd.args(&[
            "defaults",
            "read",
            &domain,
            &format!("defaults_write_local_{}", n),
        ]);
        let cmd_output = testutils::run_cmd(&mut cmd);
        assert!(cmd_output.status.success());
        assert_eq!(
            format!("{}\n", orig_check_value),
            String::from_utf8_lossy(&cmd_output.stdout)
        );
    }

    // Set the key to the new value ourselves.
    for (n, (_, _, _, new_value, _)) in test_values.iter().enumerate() {
        let mut cmd = testutils::up_cmd(&temp_dir);

        let defaults_key = format!("defaults_write_local_{}", n);
        cmd.args(&["defaults", "write", &domain, &defaults_key, new_value]);
        let cmd_output = testutils::run_cmd(&mut cmd);
        assert!(cmd_output.status.success());
        assert::contains(
            &String::from_utf8_lossy(&cmd_output.stderr),
            &format!("Changing default {} {}", domain, defaults_key),
        );
    }

    // Check that defaults agrees with the new value.
    for (n, (_, _, _, _, check_value)) in test_values.iter().enumerate() {
        let new_default = run_defaults(&["read", &domain, &format!("defaults_write_local_{}", n)]);
        assert_eq!(format!("{}\n", check_value), new_default);
    }
}
