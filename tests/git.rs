use assert_cmd::Command;
use camino::Utf8Path;
use color_eyre::Result;
use testutils::ensure_eq;
use testutils::ensure_utils;
use testutils::AssertCmdExt;

/// Make sure we can't run this without required args.
#[test]
fn test_missing_args() -> Result<()> {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();
    let mut cmd = testutils::crate_binary_cmd("up", &temp_dir)?;
    cmd.args(["git"].iter());
    cmd.assert().failure();

    Ok(())
}

/// Actually try cloning a git repository and make sure we can update.
#[test]
fn test_real_clone() -> Result<()> {
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

    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();
    let git_path = temp_dir.join("hello_world_repo");

    // Clone to directory.
    {
        up_git_cmd(&git_path, &temp_dir)?
            .assert()
            .eprint_stdout_stderr()
            .try_success()?;
        ensure_utils::file(&git_path.join("README"), "Hello World!\n")?;
        check_repo(
            &git_path,
            "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
            "master",
            "up/master",
        )?;
        ensure_eq!(
            run_git_cmd(&git_path, &["rev-parse", "up/HEAD"], true)?.trim(),
            "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
        );
    }

    // Clone again to the same directory, different branch.
    {
        up_git_cmd(&git_path, &temp_dir)?
            .args(["--branch", "test"])
            .assert()
            .eprint_stdout_stderr()
            .try_success()?;
        check_repo(
            &git_path,
            "b3cbd5bbd7e81436d2eee04537ea2b4c0cad4cdf",
            "test",
            "up/test",
        )?;
        // File from master still there.
        ensure_utils::file(&git_path.join("README"), "Hello World!\n")?;
        // File from test was added.
        ensure_utils::file(&git_path.join("CONTRIBUTING.md"), "## Contributing\n")?;
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
        )?;
        // Add a commit not on master.
        run_git_cmd(&git_path, &["merge", "--ff", "up/test"], true)?;
        // TODO(gib): change `checkout -` to `switch -` once base docker image supports
        // it. Go back to master.
        run_git_cmd(&git_path, &["checkout", "-"], true)?;
        // Reset master to previous commit.
        run_git_cmd(&git_path, &["reset", "--hard", "@^"], true)?;
        // Create a branch without an upstream (we shouldn't prune).
        run_git_cmd(&git_path, &["branch", "no_prune_no_upstream", "@"], true)?;

        // Create a branch with an upstream and no diff (we should prune).
        run_git_cmd(
            &git_path,
            &["branch", "--track", "should_be_pruned", "@"],
            true,
        )?;
        let mut cmd = up_git_cmd(&git_path, &temp_dir)?;
        cmd.args(["--branch", "test"]);
        cmd.assert().eprint_stdout_stderr().try_success()?;
        check_repo(
            &git_path,
            "b3cbd5bbd7e81436d2eee04537ea2b4c0cad4cdf",
            "test",
            "up/test",
        )?;
        // File from master still there.
        ensure_utils::file(&git_path.join("README"), "Hello World!\n")?;
        // File from test was added.
        ensure_utils::file(&git_path.join("CONTRIBUTING.md"), "## Contributing\n")?;

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
        )?;

        let mut cmd = up_git_cmd(&git_path, &temp_dir)?;
        // This time try to prune.
        cmd.args(["--branch", "test", "--prune"]);
        cmd.assert().eprint_stdout_stderr().try_success()?;
        check_repo(
            &git_path,
            "b3cbd5bbd7e81436d2eee04537ea2b4c0cad4cdf",
            "test",
            "up/test",
        )?;
        // File from master still there.
        ensure_utils::file(&git_path.join("README"), "Hello World!\n")?;
        // File from test was added.
        ensure_utils::file(&git_path.join("CONTRIBUTING.md"), "## Contributing\n")?;

        // Branch has matching remote-tracking branch so should still be there.
        run_git_cmd(
            &git_path,
            &["show-ref", "--verify", "--quiet", "refs/heads/master"],
            true,
        )?;

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
        )?;

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
        )?;

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
        )?;
    }

    Ok(())
}

fn up_git_cmd(git_path: &Utf8Path, temp_dir: &Utf8Path) -> Result<Command> {
    let mut cmd = testutils::crate_binary_cmd("up", temp_dir)?;
    cmd.args(
        [
            "git",
            "--git-url",
            "https://github.com/octocat/Hello-World",
            "--git-path",
            git_path.as_str(),
            "--remote",
            "up",
        ]
        .iter(),
    );

    Ok(cmd)
}

/// Run a `git` command to test the internal git setup works as expected.
fn run_git_cmd(git_path: &Utf8Path, args: &[&str], success: bool) -> Result<String> {
    let assert = Command::new("git")
        .args(["-C", git_path.as_str()])
        .args(args)
        .assert();
    let assert = match success {
        true => assert.eprint_stdout_stderr().try_success()?,
        false => assert.eprint_stdout_stderr().try_failure()?,
    };
    Ok(String::from_utf8_lossy(&assert.get_output().stdout).to_string())
}

fn check_repo(
    git_path: &Utf8Path,
    head_commit: &str,
    head_branch: &str,
    head_upstream: &str,
) -> Result<()> {
    ensure_eq!(
        run_git_cmd(git_path, &["rev-parse", "HEAD"], true)?.trim(),
        head_commit
    );
    ensure_eq!(
        run_git_cmd(git_path, &["rev-parse", "--abbrev-ref", "HEAD"], true)?.trim(),
        head_branch
    );
    ensure_eq!(
        run_git_cmd(
            git_path,
            &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
            true
        )?
        .trim(),
        head_upstream
    );

    Ok(())
}
