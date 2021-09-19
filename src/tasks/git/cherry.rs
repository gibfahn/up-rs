use std::{collections::HashSet, io::Read};

use color_eyre::eyre::Result;
use git2::{Branch, DiffFormat, DiffOptions, Oid, Repository, Revwalk};
use log::trace;
use ring::digest::{Context, Digest, SHA256};

use crate::tasks::git::{branch::get_branch_name, errors::GitError as E};

/// Return true if there are commits that aren't in upstream but are in head.
///
/// Find the merge-base of `upstream` and `head`, then look at the commits on
/// `head`, and checks whether there is an equivalent patch for each one in
/// `upstream`.
///
/// Equivalence is deterimined by taking the sha256sum of the patch with
/// whitespace removed, and then comparing the sha256sums.
///
/// Equivalent of `git cherry -v "$up_branch" "$branch" | grep -q '^+'`
/// Refs: <https://stackoverflow.com/questions/49480468/is-there-a-git-cherry-in-libgit2>
pub(super) fn unmerged_commits(
    repo: &Repository,
    upstream: &Branch,
    head: &Branch,
) -> Result<bool> {
    // TODO(gib): Add tests: https://github.com/git/git/blob/master/t/t3500-cherry.sh
    let head_name = get_branch_name(head)?;
    let upstream_name = get_branch_name(upstream)?;
    let head_oid = head.get().target().ok_or(E::NoOidFound {
        branch_name: head_name,
    })?;
    let upstream_oid = upstream.get().target().ok_or(E::NoOidFound {
        branch_name: upstream_name,
    })?;

    let merge_base = repo.merge_base(head_oid, upstream_oid)?;
    let upstream_ids = rev_list(repo, upstream_oid, merge_base)?;

    let mut upstream_patch_ids = HashSet::new();

    for id in upstream_ids {
        let id = id?;
        upstream_patch_ids.insert(patch_id(repo, id)?.as_ref().to_owned());
    }
    trace!("Upstream patch IDs: {:?}", &upstream_patch_ids);

    let merge_base = repo.merge_base(head_oid, upstream_oid)?;
    let head_ids: Vec<Oid> = rev_list(repo, head_oid, merge_base)?.collect::<Result<_, _>>()?;
    trace!("Found head IDs: {:?}", head_ids);

    for id in head_ids {
        let head_patch_id = patch_id(repo, id)?;
        trace!("Head patch ID for '{:?}': '{:?}'", id, head_patch_id);
        if !upstream_patch_ids.contains(head_patch_id.as_ref()) {
            // Found an unmerged commit.
            return Ok(true);
        }
    }

    // We didn't find any unmerged commits.
    Ok(false)
}

/// Generate a patch-id for the commit.
///
/// Take the sha256sum of the patch with whitespace removed, and
/// then comparing the sha256sums.
///
/// <https://git.uis.cam.ac.uk/man/git-patch-id.html>
// TODO(gib): consider running in parallel.
// TODO(gib): Add tests: https://github.com/git/git/blob/306ee63a703ad67c54ba1209dc11dd9ea500dc1f/t/t4204-patch-id.sh
fn patch_id(repo: &Repository, id: Oid) -> Result<Digest> {
    // Get commit for Oid.
    let commit = repo.find_commit(id).map_err(|e| E::NoCommitFound {
        oid: id.to_string(),
        source: e,
    })?;
    let parent = commit.parent(0)?;
    // TODO(gib): What diff options are needed? What does git set?
    let mut diff_opts = DiffOptions::new();
    // TODO(gib): Extract into parent function.
    let diff = repo.diff_tree_to_tree(
        Some(&parent.tree()?),
        Some(&commit.tree()?),
        Some(&mut diff_opts),
    )?;

    let mut trimmed_diff: Vec<u8> = Vec::new();

    // Convert diff to string so we can get the sha256sum.
    diff.print(DiffFormat::PatchId, |delta, hunk_opt, line| -> bool {
        trimmed_diff.extend(&u32_to_u8_array(delta.flags().bits()));
        if let Some(hunk) = hunk_opt {
            trimmed_diff.extend(hunk.header());
        }
        trimmed_diff.extend(line.content());
        true
    })?;

    sha256_digest(&trimmed_diff[..])
}

#[allow(clippy::cast_possible_truncation)]
const fn u32_to_u8_array(x: u32) -> [u8; 4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;

    [b1, b2, b3, b4]
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

fn rev_list(repo: &Repository, from: Oid, to: Oid) -> Result<Revwalk> {
    let mut revwalk = repo.revwalk()?;
    // TODO(gib): do I need to set a revwalk.set_sorting(Sort::REVERSE) here?
    revwalk.push(from)?;
    revwalk.hide(to)?;

    Ok(revwalk)
}
