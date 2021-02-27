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
    // Repo commit history:
    //
    // ❯ g la
    // * b1b3f97 (up/octocat-patch-1) sentence case
    // | * b3cbd5b (up/test) Create CONTRIBUTING.md
    // |/
    // * 7fd1a60 (HEAD -> master, up/master, up/HEAD) Merge pull request #6 from Spaceghost/patch-1
    // |\
    // | * 7629413 New line at end of file. --Signed off by Spaceghost
    // |/
    // * 553c207 first commit

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
        assert_eq!(
            run_git_cmd(&git_path, &["rev-parse", "up/HEAD"], true).trim(),
            "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
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
    // Also create a branch to prune and check we prune it.
    {
        // Create a branch based on master.
        run_git_cmd(
            &git_path,
            // TODO(gib): change `checkout -b` to `switch -c` once base docker image supports it.
            &[
                "checkout",
                "-b",
                "no_prune_unmerged_changes",
                "--track",
                "up/master",
            ],
            true,
        );
        // Add a commit not on master.
        run_git_cmd(&git_path, &["merge", "--ff", "up/test"], true);
        // TODO(gib): change `checkout -` to `switch -` once base docker image supports
        // it. Go back to master.
        run_git_cmd(&git_path, &["checkout", "-"], true);
        // Reset master to previous commit.
        run_git_cmd(&git_path, &["reset", "--hard", "@^"], true);
        // Create a branch without an upstream (we shouldn't prune).
        run_git_cmd(&git_path, &["branch", "no_prune_no_upstream", "@"], true);

        // Create a branch with an upstream and no diff (we should prune).
        run_git_cmd(
            &git_path,
            &["branch", "--track", "should_be_pruned", "@"],
            true,
        );
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

        // Branch shouldn't have been pruned as we didn't set the flag.
        run_git_cmd(
            &git_path,
            &[
                "show-ref",
                "--verify",
                "--quiet",
                "refs/heads/should_be_pruned",
            ],
            true,
        );

        let mut cmd = up_git_cmd(&git_path, &temp_dir);
        // This time try to prune.
        cmd.args(&["--branch", "test", "--prune"]);
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

        // Branch has matching remote-tracking branch so should still be there.
        run_git_cmd(
            &git_path,
            &["show-ref", "--verify", "--quiet", "refs/heads/master"],
            true,
        );

        // Branch has no upstream so should still be there.
        run_git_cmd(
            &git_path,
            &[
                "show-ref",
                "--verify",
                "--quiet",
                "refs/heads/no_prune_no_upstream",
            ],
            true,
        );

        // Branch has uncommitted changes so should still be there.
        run_git_cmd(
            &git_path,
            &[
                "show-ref",
                "--verify",
                "--quiet",
                "refs/heads/no_prune_unmerged_changes",
            ],
            true,
        );

        // We asked to prune so the branch should no longer be there.
        run_git_cmd(
            &git_path,
            &[
                "show-ref",
                "--verify",
                "--quiet",
                "refs/heads/should_be_pruned",
            ],
            false,
        );
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
fn run_git_cmd(git_path: &str, args: &[&str], success: bool) -> String {
    let cmd_output = testutils::run_cmd(Command::new("git").args(&["-C", git_path]).args(args));
    assert_eq!(cmd_output.status.success(), success);
    String::from_utf8_lossy(&cmd_output.stdout).to_string()
}

fn check_repo(git_path: &str, head_commit: &str, head_branch: &str, head_upstream: &str) {
    assert_eq!(
        run_git_cmd(git_path, &["rev-parse", "HEAD"], true).trim(),
        head_commit
    );
    assert_eq!(
        run_git_cmd(git_path, &["rev-parse", "--abbrev-ref", "HEAD"], true).trim(),
        head_branch
    );
    assert_eq!(
        run_git_cmd(
            git_path,
            &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
            true
        )
        .trim(),
        head_upstream
    );
}
