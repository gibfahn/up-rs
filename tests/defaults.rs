// Defaults tests are macOS only.
#![cfg(target_os = "macos")]

use duct::cmd;
use predicates::prelude::*;

#[test]
fn test_defaults_read_global() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    let mut expected_value = cmd!("defaults", "read", "-g", "com.apple.sound.beep.sound")
        .read()
        .unwrap();
    expected_value.push('\n');

    // Reading a normal value should have the same output as the defaults command (but yaml not
    // defaults own format).
    {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
        cmd.args(["defaults", "read", "-g", "com.apple.sound.beep.sound"]);
        cmd.assert().success().stdout(expected_value.clone());
    }

    // Providing a full absolute path to a plist file should also work.
    {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
        cmd.args([
            "defaults",
            "read",
            &format!(
                "{}/Library/Preferences/.GlobalPreferences.plist",
                dirs::home_dir().unwrap().display()
            ),
            "com.apple.sound.beep.sound",
        ]);
        cmd.assert().success().stdout(expected_value);
    }

    // Setting -g is the same as setting the domain NSGlobalDomain, so shouldn't pass both a key and
    // a value to `defaults read`.
    {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
        cmd.args([
            "defaults",
            "read",
            "-g",
            "NSGlobalDomain",
            "com.apple.sound.beep.sound",
        ]);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("both a domain and a key"));
    }
}

#[test]
fn test_defaults_read_local() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    // Four-letter codes for view modes: `icnv`, `clmv`, `glyv`, `Nlsv`
    let mut expected_value = cmd!(
        "defaults",
        "read",
        "com.apple.finder",
        "FXPreferredViewStyle"
    )
    .read()
    .unwrap();
    expected_value.push('\n');

    // Reading a normal value should have the same output as the defaults command (but yaml not
    // defaults own format).
    {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
        cmd.args([
            "defaults",
            "read",
            "com.apple.finder",
            "FXPreferredViewStyle",
        ]);
        cmd.assert().success().stdout(expected_value.clone());
    }

    // A .plist extension should be allowed too.
    {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
        cmd.args([
            "defaults",
            "read",
            "com.apple.finder.plist",
            "FXPreferredViewStyle",
        ]);
        cmd.assert().success().stdout(expected_value.clone());
    }

    // Providing a full absolute path to a plist file should also work.
    {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
        cmd.args([
            "defaults",
            "read",
            &format!(
                "{}/Library/Preferences/com.apple.finder.plist",
                dirs::home_dir().unwrap().display()
            ),
            "FXPreferredViewStyle",
        ]);
        cmd.assert().success().stdout(expected_value);
    }
}

#[test]
fn test_defaults_write_local() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    let domain = format!("co.fahn.up-rs.test-{}", testutils::function_path!());

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

        let defaults_key = format!("defaults_write_local_{n}");
        let defaults_type = format!("-{defaults_type}");
        let mut args = vec!["write", &domain, &defaults_key, &defaults_type];
        args.extend(values);

        // Write the original value to a test plist file.
        cmd("defaults", &args).run().unwrap();
    }

    // Check we agree with `defaults` about the original value.
    for (n, (_, _, orig_check_value, _, _)) in test_values.iter().enumerate() {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
        cmd.args([
            "defaults",
            "read",
            &domain,
            &format!("defaults_write_local_{n}"),
        ]);
        cmd.assert()
            .success()
            .stdout(format!("{orig_check_value}\n"));
    }

    // Set the key to the new value ourselves.
    for (n, (_, _, _, new_value, _)) in test_values.iter().enumerate() {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);

        let defaults_key = format!("defaults_write_local_{n}");
        cmd.args(["defaults", "write", &domain, &defaults_key, new_value]);
        cmd.assert()
            .success()
            .stderr(predicate::str::contains(format!(
                "Changing default {domain} {defaults_key}"
            )));
    }

    // Check that defaults agrees with the new value.
    for (n, (_, _, _, _, check_value)) in test_values.iter().enumerate() {
        let new_default = cmd!(
            "defaults",
            "read",
            &domain,
            &format!("defaults_write_local_{n}")
        )
        .read()
        .unwrap();
        assert_eq!(*check_value, new_default);
    }
}
