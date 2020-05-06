workspace(name = "up_rs")

load("//cargo:crates.bzl", "raze_fetch_remote_crates")

# TODO(gib): cargo-raze should add the shasums.
# DEBUG: Rule 'raze__anyhow__1_0_28' indicated that a canonical reproducible form can be obtained by modifying arguments sha256 = "d9a60d744a80c30fcb657dfe2c1b22bcb3e814c1a1e3674f32bf5820b570fbff"
raze_fetch_remote_crates()

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "bazel_skylib",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/bazel-skylib/releases/download/1.0.2/bazel-skylib-1.0.2.tar.gz",
        "https://github.com/bazelbuild/bazel-skylib/releases/download/1.0.2/bazel-skylib-1.0.2.tar.gz",
    ],
    sha256 = "97e70364e9249702246c0e9444bccdc4b847bed1eb03c5a3ece4f83dfe6abc44",
)

http_archive(
    name = "io_bazel_rules_rust",
    sha256 = "171b70bbb40d26bae3d009be5417fe2d8ab04b8ae62f17406815544b9280b2f2",
    strip_prefix = "rules_rust-8d3cb6878cf1447e81cd3d7f97057e70285fc833",
    urls = [
        # Master branch as of 2020-05-01.
        "https://github.com/bazelbuild/rules_rust/archive/8d3cb6878cf1447e81cd3d7f97057e70285fc833.tar.gz",
    ],
)

load("@io_bazel_rules_rust//rust:repositories.bzl", "rust_repositories")
rust_repositories(version = "1.43.0")
# rust_repositories(version = "nightly", iso_date = "2020-04-19")

load("@io_bazel_rules_rust//:workspace.bzl", "bazel_version")
bazel_version(name = "bazel_version")
