use std::process::Command;

use testutils::assert;

/// Make sure we can't run this without required args.
#[test]
fn missing_args() {
    let mut cmd = testutils::up_cmd();
    cmd.args(["git"].iter());
    let cmd_output = testutils::run_cmd(cmd);
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

    fn get_cmd(git_path: &str) -> Command {
        let mut cmd = testutils::up_cmd();
        cmd.args(
            [
                "git",
                "--git-url",
                "https://github.com/octocat/Hello-World",
                "--git-path",
                git_path,
            ]
            .iter(),
        );
        cmd
    }

    // Clone to directory.
    {
        let cmd_output = testutils::run_cmd(get_cmd(&git_path));
        assert_eq!(
            cmd_output.status.success(),
            true,
            "\n No args should fail the command.",
        );
        assert::file(&git_pathbuf.join("README"), "Hello World!\n");
    }

    // Clone again to the same directory.
    {
        let cmd_output = testutils::run_cmd(get_cmd(&git_path));
        assert_eq!(
            cmd_output.status.success(),
            true,
            "\n No args should fail the command.",
        );
        assert::file(&git_pathbuf.join("README"), "Hello World!\n");
    }
}
