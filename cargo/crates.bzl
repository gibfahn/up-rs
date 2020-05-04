"""
cargo-raze crate workspace functions

DO NOT EDIT! Replaced on runs of cargo-raze
"""
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")

def _new_http_archive(name, **kwargs):
    if not native.existing_rule(name):
        http_archive(name=name, **kwargs)

def _new_git_repository(name, **kwargs):
    if not native.existing_rule(name):
        new_git_repository(name=name, **kwargs)

def raze_fetch_remote_crates():

    _new_http_archive(
        name = "raze__aho_corasick__0_7_10",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/aho-corasick/aho-corasick-0.7.10.crate",
        type = "tar.gz",
        strip_prefix = "aho-corasick-0.7.10",

        build_file = Label("//cargo/remote:aho-corasick-0.7.10.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__ansi_term__0_11_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/ansi_term/ansi_term-0.11.0.crate",
        type = "tar.gz",
        strip_prefix = "ansi_term-0.11.0",

        build_file = Label("//cargo/remote:ansi_term-0.11.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__anyhow__1_0_28",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/anyhow/anyhow-1.0.28.crate",
        type = "tar.gz",
        strip_prefix = "anyhow-1.0.28",

        build_file = Label("//cargo/remote:anyhow-1.0.28.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__arc_swap__0_4_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/arc-swap/arc-swap-0.4.6.crate",
        type = "tar.gz",
        strip_prefix = "arc-swap-0.4.6",

        build_file = Label("//cargo/remote:arc-swap-0.4.6.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__arrayref__0_3_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/arrayref/arrayref-0.3.6.crate",
        type = "tar.gz",
        strip_prefix = "arrayref-0.3.6",

        build_file = Label("//cargo/remote:arrayref-0.3.6.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__arrayvec__0_5_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/arrayvec/arrayvec-0.5.1.crate",
        type = "tar.gz",
        strip_prefix = "arrayvec-0.5.1",

        build_file = Label("//cargo/remote:arrayvec-0.5.1.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__atty__0_2_14",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/atty/atty-0.2.14.crate",
        type = "tar.gz",
        strip_prefix = "atty-0.2.14",

        build_file = Label("//cargo/remote:atty-0.2.14.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__autocfg__1_0_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/autocfg/autocfg-1.0.0.crate",
        type = "tar.gz",
        strip_prefix = "autocfg-1.0.0",

        build_file = Label("//cargo/remote:autocfg-1.0.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__base64__0_11_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/base64/base64-0.11.0.crate",
        type = "tar.gz",
        strip_prefix = "base64-0.11.0",

        build_file = Label("//cargo/remote:base64-0.11.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__bitflags__1_2_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/bitflags/bitflags-1.2.1.crate",
        type = "tar.gz",
        strip_prefix = "bitflags-1.2.1",

        build_file = Label("//cargo/remote:bitflags-1.2.1.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__blake2b_simd__0_5_10",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/blake2b_simd/blake2b_simd-0.5.10.crate",
        type = "tar.gz",
        strip_prefix = "blake2b_simd-0.5.10",

        build_file = Label("//cargo/remote:blake2b_simd-0.5.10.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__bstr__0_2_12",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/bstr/bstr-0.2.12.crate",
        type = "tar.gz",
        strip_prefix = "bstr-0.2.12",

        build_file = Label("//cargo/remote:bstr-0.2.12.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__cc__1_0_52",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/cc/cc-1.0.52.crate",
        type = "tar.gz",
        strip_prefix = "cc-1.0.52",

        build_file = Label("//cargo/remote:cc-1.0.52.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__cfg_if__0_1_10",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/cfg-if/cfg-if-0.1.10.crate",
        type = "tar.gz",
        strip_prefix = "cfg-if-0.1.10",

        build_file = Label("//cargo/remote:cfg-if-0.1.10.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__chrono__0_4_11",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/chrono/chrono-0.4.11.crate",
        type = "tar.gz",
        strip_prefix = "chrono-0.4.11",

        build_file = Label("//cargo/remote:chrono-0.4.11.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__clap__2_33_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/clap/clap-2.33.0.crate",
        type = "tar.gz",
        strip_prefix = "clap-2.33.0",

        build_file = Label("//cargo/remote:clap-2.33.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__constant_time_eq__0_1_5",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/constant_time_eq/constant_time_eq-0.1.5.crate",
        type = "tar.gz",
        strip_prefix = "constant_time_eq-0.1.5",

        build_file = Label("//cargo/remote:constant_time_eq-0.1.5.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__crossbeam__0_7_3",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/crossbeam/crossbeam-0.7.3.crate",
        type = "tar.gz",
        strip_prefix = "crossbeam-0.7.3",

        build_file = Label("//cargo/remote:crossbeam-0.7.3.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__crossbeam_channel__0_4_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/crossbeam-channel/crossbeam-channel-0.4.2.crate",
        type = "tar.gz",
        strip_prefix = "crossbeam-channel-0.4.2",

        build_file = Label("//cargo/remote:crossbeam-channel-0.4.2.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__crossbeam_deque__0_7_3",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/crossbeam-deque/crossbeam-deque-0.7.3.crate",
        type = "tar.gz",
        strip_prefix = "crossbeam-deque-0.7.3",

        build_file = Label("//cargo/remote:crossbeam-deque-0.7.3.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__crossbeam_epoch__0_8_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/crossbeam-epoch/crossbeam-epoch-0.8.2.crate",
        type = "tar.gz",
        strip_prefix = "crossbeam-epoch-0.8.2",

        build_file = Label("//cargo/remote:crossbeam-epoch-0.8.2.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__crossbeam_queue__0_2_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/crossbeam-queue/crossbeam-queue-0.2.1.crate",
        type = "tar.gz",
        strip_prefix = "crossbeam-queue-0.2.1",

        build_file = Label("//cargo/remote:crossbeam-queue-0.2.1.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__crossbeam_utils__0_7_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/crossbeam-utils/crossbeam-utils-0.7.2.crate",
        type = "tar.gz",
        strip_prefix = "crossbeam-utils-0.7.2",

        build_file = Label("//cargo/remote:crossbeam-utils-0.7.2.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__dirs__2_0_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/dirs/dirs-2.0.2.crate",
        type = "tar.gz",
        strip_prefix = "dirs-2.0.2",

        build_file = Label("//cargo/remote:dirs-2.0.2.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__dirs_sys__0_3_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/dirs-sys/dirs-sys-0.3.4.crate",
        type = "tar.gz",
        strip_prefix = "dirs-sys-0.3.4",

        build_file = Label("//cargo/remote:dirs-sys-0.3.4.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__displaydoc__0_1_5",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/displaydoc/displaydoc-0.1.5.crate",
        type = "tar.gz",
        strip_prefix = "displaydoc-0.1.5",

        build_file = Label("//cargo/remote:displaydoc-0.1.5.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__dtoa__0_4_5",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/dtoa/dtoa-0.4.5.crate",
        type = "tar.gz",
        strip_prefix = "dtoa-0.4.5",

        build_file = Label("//cargo/remote:dtoa-0.4.5.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__envy__0_4_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/envy/envy-0.4.1.crate",
        type = "tar.gz",
        strip_prefix = "envy-0.4.1",

        build_file = Label("//cargo/remote:envy-0.4.1.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__fnv__1_0_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/fnv/fnv-1.0.6.crate",
        type = "tar.gz",
        strip_prefix = "fnv-1.0.6",

        build_file = Label("//cargo/remote:fnv-1.0.6.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__getrandom__0_1_14",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/getrandom/getrandom-0.1.14.crate",
        type = "tar.gz",
        strip_prefix = "getrandom-0.1.14",

        build_file = Label("//cargo/remote:getrandom-0.1.14.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__git2__0_13_5",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/git2/git2-0.13.5.crate",
        type = "tar.gz",
        strip_prefix = "git2-0.13.5",

        build_file = Label("//cargo/remote:git2-0.13.5.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__globset__0_4_5",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/globset/globset-0.4.5.crate",
        type = "tar.gz",
        strip_prefix = "globset-0.4.5",

        build_file = Label("//cargo/remote:globset-0.4.5.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__heck__0_3_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/heck/heck-0.3.1.crate",
        type = "tar.gz",
        strip_prefix = "heck-0.3.1",

        build_file = Label("//cargo/remote:heck-0.3.1.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__hermit_abi__0_1_12",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/hermit-abi/hermit-abi-0.1.12.crate",
        type = "tar.gz",
        strip_prefix = "hermit-abi-0.1.12",

        build_file = Label("//cargo/remote:hermit-abi-0.1.12.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__idna__0_2_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/idna/idna-0.2.0.crate",
        type = "tar.gz",
        strip_prefix = "idna-0.2.0",

        build_file = Label("//cargo/remote:idna-0.2.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__ignore__0_4_14",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/ignore/ignore-0.4.14.crate",
        type = "tar.gz",
        strip_prefix = "ignore-0.4.14",

        build_file = Label("//cargo/remote:ignore-0.4.14.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__itoa__0_4_5",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/itoa/itoa-0.4.5.crate",
        type = "tar.gz",
        strip_prefix = "itoa-0.4.5",

        build_file = Label("//cargo/remote:itoa-0.4.5.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__jobserver__0_1_21",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/jobserver/jobserver-0.1.21.crate",
        type = "tar.gz",
        strip_prefix = "jobserver-0.1.21",

        build_file = Label("//cargo/remote:jobserver-0.1.21.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__lazy_static__1_4_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/lazy_static/lazy_static-1.4.0.crate",
        type = "tar.gz",
        strip_prefix = "lazy_static-1.4.0",

        build_file = Label("//cargo/remote:lazy_static-1.4.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__libc__0_2_69",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/libc/libc-0.2.69.crate",
        type = "tar.gz",
        strip_prefix = "libc-0.2.69",

        build_file = Label("//cargo/remote:libc-0.2.69.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__libgit2_sys__0_12_5_1_0_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/libgit2-sys/libgit2-sys-0.12.5+1.0.0.crate",
        type = "tar.gz",
        strip_prefix = "libgit2-sys-0.12.5+1.0.0",

        build_file = Label("//cargo/remote:libgit2-sys-0.12.5+1.0.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__libssh2_sys__0_2_17",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/libssh2-sys/libssh2-sys-0.2.17.crate",
        type = "tar.gz",
        strip_prefix = "libssh2-sys-0.2.17",

        build_file = Label("//cargo/remote:libssh2-sys-0.2.17.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__libz_sys__1_0_25",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/libz-sys/libz-sys-1.0.25.crate",
        type = "tar.gz",
        strip_prefix = "libz-sys-1.0.25",

        build_file = Label("//cargo/remote:libz-sys-1.0.25.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__linked_hash_map__0_5_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/linked-hash-map/linked-hash-map-0.5.2.crate",
        type = "tar.gz",
        strip_prefix = "linked-hash-map-0.5.2",

        build_file = Label("//cargo/remote:linked-hash-map-0.5.2.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__log__0_4_8",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/log/log-0.4.8.crate",
        type = "tar.gz",
        strip_prefix = "log-0.4.8",

        build_file = Label("//cargo/remote:log-0.4.8.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__matches__0_1_8",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/matches/matches-0.1.8.crate",
        type = "tar.gz",
        strip_prefix = "matches-0.1.8",

        build_file = Label("//cargo/remote:matches-0.1.8.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__maybe_uninit__2_0_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/maybe-uninit/maybe-uninit-2.0.0.crate",
        type = "tar.gz",
        strip_prefix = "maybe-uninit-2.0.0",

        build_file = Label("//cargo/remote:maybe-uninit-2.0.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__memchr__2_3_3",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/memchr/memchr-2.3.3.crate",
        type = "tar.gz",
        strip_prefix = "memchr-2.3.3",

        build_file = Label("//cargo/remote:memchr-2.3.3.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__memoffset__0_5_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/memoffset/memoffset-0.5.4.crate",
        type = "tar.gz",
        strip_prefix = "memoffset-0.5.4",

        build_file = Label("//cargo/remote:memoffset-0.5.4.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__num_integer__0_1_42",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-integer/num-integer-0.1.42.crate",
        type = "tar.gz",
        strip_prefix = "num-integer-0.1.42",

        build_file = Label("//cargo/remote:num-integer-0.1.42.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__num_traits__0_2_11",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-traits/num-traits-0.2.11.crate",
        type = "tar.gz",
        strip_prefix = "num-traits-0.2.11",

        build_file = Label("//cargo/remote:num-traits-0.2.11.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__openssl_probe__0_1_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/openssl-probe/openssl-probe-0.1.2.crate",
        type = "tar.gz",
        strip_prefix = "openssl-probe-0.1.2",

        build_file = Label("//cargo/remote:openssl-probe-0.1.2.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__openssl_src__111_9_0_1_1_1g",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/openssl-src/openssl-src-111.9.0+1.1.1g.crate",
        type = "tar.gz",
        strip_prefix = "openssl-src-111.9.0+1.1.1g",

        build_file = Label("//cargo/remote:openssl-src-111.9.0+1.1.1g.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__openssl_sys__0_9_55",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/openssl-sys/openssl-sys-0.9.55.crate",
        type = "tar.gz",
        strip_prefix = "openssl-sys-0.9.55",

        build_file = Label("//cargo/remote:openssl-sys-0.9.55.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__percent_encoding__2_1_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/percent-encoding/percent-encoding-2.1.0.crate",
        type = "tar.gz",
        strip_prefix = "percent-encoding-2.1.0",

        build_file = Label("//cargo/remote:percent-encoding-2.1.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__pkg_config__0_3_17",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/pkg-config/pkg-config-0.3.17.crate",
        type = "tar.gz",
        strip_prefix = "pkg-config-0.3.17",

        build_file = Label("//cargo/remote:pkg-config-0.3.17.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__proc_macro_error__1_0_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/proc-macro-error/proc-macro-error-1.0.2.crate",
        type = "tar.gz",
        strip_prefix = "proc-macro-error-1.0.2",

        build_file = Label("//cargo/remote:proc-macro-error-1.0.2.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__proc_macro_error_attr__1_0_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/proc-macro-error-attr/proc-macro-error-attr-1.0.2.crate",
        type = "tar.gz",
        strip_prefix = "proc-macro-error-attr-1.0.2",

        build_file = Label("//cargo/remote:proc-macro-error-attr-1.0.2.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__proc_macro2__1_0_10",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/proc-macro2/proc-macro2-1.0.10.crate",
        type = "tar.gz",
        strip_prefix = "proc-macro2-1.0.10",

        build_file = Label("//cargo/remote:proc-macro2-1.0.10.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__quote__1_0_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/quote/quote-1.0.4.crate",
        type = "tar.gz",
        strip_prefix = "quote-1.0.4",

        build_file = Label("//cargo/remote:quote-1.0.4.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__redox_syscall__0_1_56",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/redox_syscall/redox_syscall-0.1.56.crate",
        type = "tar.gz",
        strip_prefix = "redox_syscall-0.1.56",

        build_file = Label("//cargo/remote:redox_syscall-0.1.56.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__redox_users__0_3_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/redox_users/redox_users-0.3.4.crate",
        type = "tar.gz",
        strip_prefix = "redox_users-0.3.4",

        build_file = Label("//cargo/remote:redox_users-0.3.4.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__regex__1_3_7",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/regex/regex-1.3.7.crate",
        type = "tar.gz",
        strip_prefix = "regex-1.3.7",

        build_file = Label("//cargo/remote:regex-1.3.7.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__regex_syntax__0_6_17",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/regex-syntax/regex-syntax-0.6.17.crate",
        type = "tar.gz",
        strip_prefix = "regex-syntax-0.6.17",

        build_file = Label("//cargo/remote:regex-syntax-0.6.17.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__rust_argon2__0_7_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/rust-argon2/rust-argon2-0.7.0.crate",
        type = "tar.gz",
        strip_prefix = "rust-argon2-0.7.0",

        build_file = Label("//cargo/remote:rust-argon2-0.7.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__ryu__1_0_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/ryu/ryu-1.0.4.crate",
        type = "tar.gz",
        strip_prefix = "ryu-1.0.4",

        build_file = Label("//cargo/remote:ryu-1.0.4.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__same_file__1_0_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/same-file/same-file-1.0.6.crate",
        type = "tar.gz",
        strip_prefix = "same-file-1.0.6",

        build_file = Label("//cargo/remote:same-file-1.0.6.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__scopeguard__1_1_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/scopeguard/scopeguard-1.1.0.crate",
        type = "tar.gz",
        strip_prefix = "scopeguard-1.1.0",

        build_file = Label("//cargo/remote:scopeguard-1.1.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__serde__1_0_106",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/serde/serde-1.0.106.crate",
        type = "tar.gz",
        strip_prefix = "serde-1.0.106",

        build_file = Label("//cargo/remote:serde-1.0.106.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__serde_derive__1_0_106",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/serde_derive/serde_derive-1.0.106.crate",
        type = "tar.gz",
        strip_prefix = "serde_derive-1.0.106",

        build_file = Label("//cargo/remote:serde_derive-1.0.106.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__serde_json__1_0_52",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/serde_json/serde_json-1.0.52.crate",
        type = "tar.gz",
        strip_prefix = "serde_json-1.0.52",

        build_file = Label("//cargo/remote:serde_json-1.0.52.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__serde_yaml__0_8_11",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/serde_yaml/serde_yaml-0.8.11.crate",
        type = "tar.gz",
        strip_prefix = "serde_yaml-0.8.11",

        build_file = Label("//cargo/remote:serde_yaml-0.8.11.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__shellexpand__2_0_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/shellexpand/shellexpand-2.0.0.crate",
        type = "tar.gz",
        strip_prefix = "shellexpand-2.0.0",

        build_file = Label("//cargo/remote:shellexpand-2.0.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__slog__2_5_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/slog/slog-2.5.2.crate",
        type = "tar.gz",
        strip_prefix = "slog-2.5.2",

        build_file = Label("//cargo/remote:slog-2.5.2.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__slog_async__2_5_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/slog-async/slog-async-2.5.0.crate",
        type = "tar.gz",
        strip_prefix = "slog-async-2.5.0",

        build_file = Label("//cargo/remote:slog-async-2.5.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__slog_scope__4_3_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/slog-scope/slog-scope-4.3.0.crate",
        type = "tar.gz",
        strip_prefix = "slog-scope-4.3.0",

        build_file = Label("//cargo/remote:slog-scope-4.3.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__slog_stdlog__4_0_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/slog-stdlog/slog-stdlog-4.0.0.crate",
        type = "tar.gz",
        strip_prefix = "slog-stdlog-4.0.0",

        build_file = Label("//cargo/remote:slog-stdlog-4.0.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__slog_term__2_5_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/slog-term/slog-term-2.5.0.crate",
        type = "tar.gz",
        strip_prefix = "slog-term-2.5.0",

        build_file = Label("//cargo/remote:slog-term-2.5.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__smallvec__1_4_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/smallvec/smallvec-1.4.0.crate",
        type = "tar.gz",
        strip_prefix = "smallvec-1.4.0",

        build_file = Label("//cargo/remote:smallvec-1.4.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__strsim__0_8_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/strsim/strsim-0.8.0.crate",
        type = "tar.gz",
        strip_prefix = "strsim-0.8.0",

        build_file = Label("//cargo/remote:strsim-0.8.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__structopt__0_3_14",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/structopt/structopt-0.3.14.crate",
        type = "tar.gz",
        strip_prefix = "structopt-0.3.14",

        build_file = Label("//cargo/remote:structopt-0.3.14.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__structopt_derive__0_4_7",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/structopt-derive/structopt-derive-0.4.7.crate",
        type = "tar.gz",
        strip_prefix = "structopt-derive-0.4.7",

        build_file = Label("//cargo/remote:structopt-derive-0.4.7.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__syn__1_0_18",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/syn/syn-1.0.18.crate",
        type = "tar.gz",
        strip_prefix = "syn-1.0.18",

        build_file = Label("//cargo/remote:syn-1.0.18.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__syn_mid__0_5_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/syn-mid/syn-mid-0.5.0.crate",
        type = "tar.gz",
        strip_prefix = "syn-mid-0.5.0",

        build_file = Label("//cargo/remote:syn-mid-0.5.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__take_mut__0_2_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/take_mut/take_mut-0.2.2.crate",
        type = "tar.gz",
        strip_prefix = "take_mut-0.2.2",

        build_file = Label("//cargo/remote:take_mut-0.2.2.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__term__0_6_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/term/term-0.6.1.crate",
        type = "tar.gz",
        strip_prefix = "term-0.6.1",

        build_file = Label("//cargo/remote:term-0.6.1.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__textwrap__0_11_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/textwrap/textwrap-0.11.0.crate",
        type = "tar.gz",
        strip_prefix = "textwrap-0.11.0",

        build_file = Label("//cargo/remote:textwrap-0.11.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__thiserror__1_0_16",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/thiserror/thiserror-1.0.16.crate",
        type = "tar.gz",
        strip_prefix = "thiserror-1.0.16",

        build_file = Label("//cargo/remote:thiserror-1.0.16.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__thiserror_impl__1_0_16",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/thiserror-impl/thiserror-impl-1.0.16.crate",
        type = "tar.gz",
        strip_prefix = "thiserror-impl-1.0.16",

        build_file = Label("//cargo/remote:thiserror-impl-1.0.16.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__thread_local__1_0_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/thread_local/thread_local-1.0.1.crate",
        type = "tar.gz",
        strip_prefix = "thread_local-1.0.1",

        build_file = Label("//cargo/remote:thread_local-1.0.1.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__time__0_1_43",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/time/time-0.1.43.crate",
        type = "tar.gz",
        strip_prefix = "time-0.1.43",

        build_file = Label("//cargo/remote:time-0.1.43.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__toml__0_5_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/toml/toml-0.5.6.crate",
        type = "tar.gz",
        strip_prefix = "toml-0.5.6",

        build_file = Label("//cargo/remote:toml-0.5.6.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__unicode_bidi__0_3_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/unicode-bidi/unicode-bidi-0.3.4.crate",
        type = "tar.gz",
        strip_prefix = "unicode-bidi-0.3.4",

        build_file = Label("//cargo/remote:unicode-bidi-0.3.4.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__unicode_normalization__0_1_12",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/unicode-normalization/unicode-normalization-0.1.12.crate",
        type = "tar.gz",
        strip_prefix = "unicode-normalization-0.1.12",

        build_file = Label("//cargo/remote:unicode-normalization-0.1.12.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__unicode_segmentation__1_6_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/unicode-segmentation/unicode-segmentation-1.6.0.crate",
        type = "tar.gz",
        strip_prefix = "unicode-segmentation-1.6.0",

        build_file = Label("//cargo/remote:unicode-segmentation-1.6.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__unicode_width__0_1_7",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/unicode-width/unicode-width-0.1.7.crate",
        type = "tar.gz",
        strip_prefix = "unicode-width-0.1.7",

        build_file = Label("//cargo/remote:unicode-width-0.1.7.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__unicode_xid__0_2_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/unicode-xid/unicode-xid-0.2.0.crate",
        type = "tar.gz",
        strip_prefix = "unicode-xid-0.2.0",

        build_file = Label("//cargo/remote:unicode-xid-0.2.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__url__2_1_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/url/url-2.1.1.crate",
        type = "tar.gz",
        strip_prefix = "url-2.1.1",

        build_file = Label("//cargo/remote:url-2.1.1.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__vcpkg__0_2_8",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/vcpkg/vcpkg-0.2.8.crate",
        type = "tar.gz",
        strip_prefix = "vcpkg-0.2.8",

        build_file = Label("//cargo/remote:vcpkg-0.2.8.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__vec_map__0_8_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/vec_map/vec_map-0.8.1.crate",
        type = "tar.gz",
        strip_prefix = "vec_map-0.8.1",

        build_file = Label("//cargo/remote:vec_map-0.8.1.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__version_check__0_9_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/version_check/version_check-0.9.1.crate",
        type = "tar.gz",
        strip_prefix = "version_check-0.9.1",

        build_file = Label("//cargo/remote:version_check-0.9.1.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__walkdir__2_3_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/walkdir/walkdir-2.3.1.crate",
        type = "tar.gz",
        strip_prefix = "walkdir-2.3.1",

        build_file = Label("//cargo/remote:walkdir-2.3.1.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__wasi__0_9_0_wasi_snapshot_preview1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/wasi/wasi-0.9.0+wasi-snapshot-preview1.crate",
        type = "tar.gz",
        strip_prefix = "wasi-0.9.0+wasi-snapshot-preview1",

        build_file = Label("//cargo/remote:wasi-0.9.0+wasi-snapshot-preview1.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__winapi__0_3_8",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/winapi/winapi-0.3.8.crate",
        type = "tar.gz",
        strip_prefix = "winapi-0.3.8",

        build_file = Label("//cargo/remote:winapi-0.3.8.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__winapi_i686_pc_windows_gnu__0_4_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/winapi-i686-pc-windows-gnu/winapi-i686-pc-windows-gnu-0.4.0.crate",
        type = "tar.gz",
        strip_prefix = "winapi-i686-pc-windows-gnu-0.4.0",

        build_file = Label("//cargo/remote:winapi-i686-pc-windows-gnu-0.4.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__winapi_util__0_1_5",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/winapi-util/winapi-util-0.1.5.crate",
        type = "tar.gz",
        strip_prefix = "winapi-util-0.1.5",

        build_file = Label("//cargo/remote:winapi-util-0.1.5.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__winapi_x86_64_pc_windows_gnu__0_4_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/winapi-x86_64-pc-windows-gnu/winapi-x86_64-pc-windows-gnu-0.4.0.crate",
        type = "tar.gz",
        strip_prefix = "winapi-x86_64-pc-windows-gnu-0.4.0",

        build_file = Label("//cargo/remote:winapi-x86_64-pc-windows-gnu-0.4.0.BUILD.bazel"),
    )

    _new_http_archive(
        name = "raze__yaml_rust__0_4_3",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/yaml-rust/yaml-rust-0.4.3.crate",
        type = "tar.gz",
        strip_prefix = "yaml-rust-0.4.3",

        build_file = Label("//cargo/remote:yaml-rust-0.4.3.BUILD.bazel"),
    )

