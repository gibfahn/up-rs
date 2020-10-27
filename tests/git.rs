use std::{path::Path, process::Command};

use testutils::assert;

/// Make sure we can't run this without required args.
#[test]
fn missing_args() {
    let temp_dir = testutils::temp_dir(file!(), "missing_args").unwrap();
    let mut cmd = testutils::up_cmd(&temp_dir);
    cmd.args(["git"].iter());
    let cmd_output = testutils::run_cmd(&mut cmd);
    assert_eq!(
        cmd_output.status.success(),
        false,
        "\n No args should fail the command.",
    );
}

/// Actually try cloning a git repository and make sure we can update.
#[test]
fn real_clone() {
    let temp_dir = testutils::temp_dir(file!(), "real_clone").unwrap();
    let git_pathbuf = temp_dir.join("hello_world_repo");
    let git_path = git_pathbuf.to_string_lossy();

    // Clone to directory.
    {
        let cmd_output = testutils::run_cmd(&mut up_git_cmd(&git_path, &temp_dir));
        assert_eq!(cmd_output.status.success(), true,);
        assert::file(&git_pathbuf.join("README"), "Hello World!\n");
        check_repo(
            &git_path,
            "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
            "master",
            "up/master",
        );
    }

    // Clone again to the same directory, different branch.
    {
        let cmd_output =
            testutils::run_cmd(&mut up_git_cmd(&git_path, &temp_dir).args(&["--branch", "test"]));
        assert_eq!(cmd_output.status.success(), true,);
        check_repo(
            &git_path,
            "b3cbd5bbd7e81436d2eee04537ea2b4c0cad4cdf",
            "test",
            "up/test",
        );
        // File from master still there.
        assert::file(&git_pathbuf.join("README"), "Hello World!\n");
        // File from test was added.
        assert::file(&git_pathbuf.join("CONTRIBUTING.md"), "## Contributing\n");
    }

    // Reset head backwards and check if fast-forwards
    {
        run_git_cmd(&git_path, &["reset", "--hard", "@^"]);
        let mut cmd = up_git_cmd(&git_path, &temp_dir);
        cmd.args(&["--branch", "test"]);
        let cmd_output = testutils::run_cmd(&mut cmd);
        assert_eq!(cmd_output.status.success(), true,);
        check_repo(
            &git_path,
            "b3cbd5bbd7e81436d2eee04537ea2b4c0cad4cdf",
            "test",
            "up/test",
        );
        // File from master still there.
        assert::file(&git_pathbuf.join("README"), "Hello World!\n");
        // File from test was added.
        assert::file(&git_pathbuf.join("CONTRIBUTING.md"), "## Contributing\n");
    }
}

fn up_git_cmd(git_path: &str, temp_dir: &Path) -> Command {
    let mut cmd = testutils::up_cmd(&temp_dir);
    cmd.args(
        [
            "git",
            "--git-url",
            "https://github.com/octocat/Hello-World",
            "--git-path",
            git_path,
            "--remote",
            "up",
        ]
        .iter(),
    );
    cmd
}

/// Run a `git` command to test the internal git setup works as expected.
fn run_git_cmd(git_path: &str, args: &[&str]) -> String {
    let cmd_output = testutils::run_cmd(Command::new("git").args(&["-C", git_path]).args(args));
    assert_eq!(cmd_output.status.success(), true);
    String::from_utf8_lossy(&cmd_output.stdout).to_string()
}

fn check_repo(git_path: &str, head_commit: &str, head_branch: &str, head_upstream: &str) {
    assert_eq!(
        run_git_cmd(git_path, &["rev-parse", "HEAD"]).trim(),
        head_commit
    );
    assert_eq!(
        run_git_cmd(git_path, &["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        head_branch
    );
    assert_eq!(
        run_git_cmd(
            git_path,
            &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"]
        )
        .trim(),
        head_upstream
    );
}
