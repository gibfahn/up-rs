// Defaults tests are macOS only.
#![cfg(target_os = "macos")]

use duct::cmd;
use duct::Expression;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use test_log::test;
use tracing::debug;
use tracing::info;
use up_rs::exec::LivDuct;

/**
Key that is in the global plist on a newly setup machine, and that has the same value as yaml and as returned by the `defaults read` command.
*/
const GLOBAL_KEY: &str = "com.apple.springing.delay";

#[test]
fn test_defaults_read_global() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    let mut expected_value = cmd!("defaults", "read", "-g", GLOBAL_KEY).read().unwrap();
    expected_value.push('\n');

    // Reading a normal value should have the same output as the defaults command (but yaml not
    // defaults own format).
    {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
        cmd.args(["defaults", "read", "-g", GLOBAL_KEY]);
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
            GLOBAL_KEY,
        ]);
        cmd.assert().success().stdout(expected_value);
    }

    // Setting -g is the same as setting the domain NSGlobalDomain, so shouldn't pass both a key and
    // a value to `defaults read`.
    {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
        cmd.args(["defaults", "read", "-g", "NSGlobalDomain", GLOBAL_KEY]);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("both a domain and a key"));
    }
}

#[test]
fn test_defaults_read_local() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    // Dock region, e.g. 'GB'
    let mut expected_value = cmd!("defaults", "read", "com.apple.dock", "region")
        .read()
        .unwrap();
    expected_value.push('\n');

    // Reading a normal value should have the same output as the defaults command (but yaml not
    // defaults own format).
    {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
        cmd.args(["defaults", "read", "com.apple.dock", "region"]);
        cmd.assert().success().stdout(expected_value.clone());
    }

    // A .plist extension should be allowed too.
    {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
        cmd.args(["defaults", "read", "com.apple.dock.plist", "region"]);
        cmd.assert().success().stdout(expected_value.clone());
    }

    // Providing a full absolute path to a plist file should also work.
    {
        let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
        cmd.args([
            "defaults",
            "read",
            &format!(
                "{}/Library/Preferences/com.apple.dock.plist",
                dirs::home_dir().unwrap().display()
            ),
            "region",
        ]);
        cmd.assert().success().stdout(expected_value);
    }
}

#[derive(Debug, Clone)]
struct TestCase {
    name: &'static str,
    /// Type flags from `man defaults`, without the initial `-`.
    defaults_type: &'static str,
    /// Original value in defaults format (as used by `defaults write`). Space-separate dicts and
    /// arrays.
    orig_defaults_set_value: &'static str,
    /// Original value in yaml format (as returned by `up defaults read`).
    orig_up_check_value: &'static str,
    /// New value in yaml format (as used by `up defaults write`).
    up_set_value: &'static str,
    /// New value in defaults format (as used by `defaults read`).
    defaults_check_value: &'static str,
}

#[test]
fn test_defaults_write_local() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    let domain = format!("co.fahn.up-rs.test-{}", testutils::function_path!());

    let test_values = [
        TestCase {
            name: "basic_bool",
            defaults_type: "bool",
            orig_defaults_set_value: "true",
            orig_up_check_value: "true",
            up_set_value: "false",
            defaults_check_value: "0",
        },
        TestCase {
            name: "basic_int",
            defaults_type: "int",
            orig_defaults_set_value: "5",
            orig_up_check_value: "5",
            up_set_value: "10",
            defaults_check_value: "10",
        },
        TestCase {
            name: "basic_float",
            defaults_type: "float",
            orig_defaults_set_value: "5.123",
            orig_up_check_value: "5.123000144958496",
            up_set_value: "7.8",
            defaults_check_value: "7.8",
        },
        TestCase {
            name: "basic_string",
            defaults_type: "string",
            orig_defaults_set_value: "initial basic_string value\nline 2",
            orig_up_check_value: "|-\n  initial basic_string value\n  line 2",
            up_set_value: r#""new value\nnew line 2""#,
            defaults_check_value: "new value\nnew line 2",
        },
        // Check swapping an array to a dict works.
        TestCase {
            name: "array_to_dict",
            defaults_type: "array",
            orig_defaults_set_value: "(aarray_to_dict, barray_to_dict, carray_to_dict)",
            orig_up_check_value: "- aarray_to_dict\n- barray_to_dict\n- carray_to_dict",
            up_set_value: "x: fourarray_to_dict\ny: fivearray_to_dict\nz: sixarray_to_dict",
            defaults_check_value: r#"{
    x = "fourarray_to_dict";
    y = "fivearray_to_dict";
    z = "sixarray_to_dict";
}"#,
        },
        // Check swapping an array to a dict works.
        TestCase {
            name: "array_to_dict_2",
            defaults_type: "array",
            orig_defaults_set_value: "(d, e, f)",
            orig_up_check_value: "- d\n- e\n- f",
            up_set_value: "x: four\ny: five\nz: six",
            defaults_check_value: "{\n    x = four;\n    y = five;\n    z = six;\n}",
        },
        // Check swapping a dict to an array works.
        TestCase {
            name: "dict_to_array",
            defaults_type: "dict",
            orig_defaults_set_value: "{ u = one; v = two; w = three; }",
            orig_up_check_value: "u: one\nv: two\nw: three",
            up_set_value: "['d','e','f']",
            defaults_check_value: "(\n    d,\n    e,\n    f\n)",
        },
        // Check preserving original array values works.
        TestCase {
            name: "array_preserve_original",
            defaults_type: "array",
            orig_defaults_set_value: "(a, foo, b, bar, c)",
            orig_up_check_value: "- a\n- foo\n- b\n- bar\n- c",
            up_set_value: "['foo', '...', 'bar', 'baz']",
            defaults_check_value: "(\n    foo,\n    a,\n    b,\n    bar,\n    c,\n    baz\n)",
        },
        // Check preserving original dict values works.
        TestCase {
            name: "dict_preserve_original",
            defaults_type: "dict",
            orig_defaults_set_value: "{ a = 1; b = 3; bar = 4; c = 5; foo = 2; }",
            orig_up_check_value: "foo: '2'\nb: '3'\nbar: '4'\nc: '5'\na: '1'",
            up_set_value: "{'foo': 6, '...':'...', 'bar': 7, 'baz': 8}",
            defaults_check_value: "{\n    a = 1;\n    b = 3;\n    bar = 4;\n    baz = 8;\n    c = \
                                   5;\n    foo = 6;\n}",
        },
        // Check preserving dicts in an array of dicts works. See `defaults read -g
        // NSUserDictionaryReplacementItems`.
        TestCase {
            name: "array_of_dict_preserve_original",
            defaults_type: "array",
            orig_defaults_set_value: r#"
(
    {
        on = 1;
        replace = omw;
        with = "On my way!";
    },
    {
        on = 1;
        replace = zss;
        with = "Use \\U2318-\\U21e7-Enter.";
    },
    {
        on = 1;
        replace = za;
        with = "https://apple.com/";
    }
)
"#,
            orig_up_check_value: r"- replace: omw
  with: On my way!
  on: '1'
- replace: zss
  with: Use \U2318-\U21e7-Enter.
  on: '1'
- replace: za
  with: https://apple.com/
  on: '1'",
            up_set_value: r"
- replace: omw
  with: Newer on my way
  on: '1'
- ...
- replace: atm
  with: at the moment
  on: 1
- replace: zss
  with: Use \U2318-\U21e7-Enter.
  on: '1'
",

            defaults_check_value: r#"(
        {
        on = 1;
        replace = omw;
        with = "Newer on my way";
    },
        {
        on = 1;
        replace = omw;
        with = "On my way!";
    },
        {
        on = 1;
        replace = zss;
        with = "Use \\\\U2318-\\\\U21e7-Enter.";
    },
        {
        on = 1;
        replace = za;
        with = "https://apple.com/";
    },
        {
        on = 1;
        replace = atm;
        with = "at the moment";
    }
)"#,
        },
    ];

    for test_case in test_values {
        let TestCase {
            name,
            defaults_type,
            orig_defaults_set_value,
            orig_up_check_value,
            up_set_value,
            defaults_check_value,
        } = test_case;

        info!("Testing default {name}: {test_case:#?}");

        {
            debug!("Writing original value for {name}");

            let defaults_key = format!("defaults_write_local_{name}");
            let mut args = vec!["write", &domain, &defaults_key];

            let defaults_type_arg;
            match defaults_type {
                "array" | "dict" => (),
                _ => {
                    defaults_type_arg = format!("-{defaults_type}");
                    args.push(&defaults_type_arg);
                }
            };
            args.push(orig_defaults_set_value);

            // Write the original value to a test plist file.
            cmd("defaults", &args)
                .run_with(Expression::stdout_to_stderr)
                .unwrap();
        }

        {
            debug!("Checking we agree with `defaults` about the original value.");
            let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
            cmd.args([
                "defaults",
                "read",
                &domain,
                &format!("defaults_write_local_{name}"),
            ]);
            cmd.assert()
                .success()
                .stdout(predicate::str::diff(format!("{orig_up_check_value}\n")));
        }

        {
            debug!("Setting the key to the new value ourselves:");
            let mut cmd = testutils::test_binary_cmd("up", &temp_dir);

            let defaults_key = format!("defaults_write_local_{name}");
            cmd.args(["defaults", "write", &domain, &defaults_key, up_set_value]);
            cmd.assert()
                .success()
                .stderr(predicate::str::contains(format!(
                    "Changing default {domain} {defaults_key}"
                )));
        }

        {
            debug!("Checking that defaults agrees with the new value:");
            let new_default = cmd!(
                "defaults",
                "read",
                &domain,
                &format!("defaults_write_local_{name}")
            )
            .read()
            .unwrap();
            assert_eq!(*defaults_check_value, new_default);
        }
    }
}
