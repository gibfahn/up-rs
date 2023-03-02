# Up-rs Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.13.5](https://github.com/gibfahn/up-rs/releases/tag/0.13.5) (2023-03-02)

### Bug Fixes
- Link to libgit2 statically, instead of relying on a libgit2 library being present on the system.

## [0.13.4](https://github.com/gibfahn/up-rs/releases/tag/0.13.4) (2023-03-01)

### Bug Fixes
- defaults: create defaults parent dir if it doesn't exist ([3dea4f4](https://github.com/gibfahn/up-rs/commit/3dea4f4594657cfc4803975b1afb1a0a22def4db))
- generate: update to new serde yaml format ([97625b0](https://github.com/gibfahn/up-rs/commit/97625b0d3a905386af2133bee6d371bcf6b94168))
- git: fix double-quoting in git_path log messages ([c607fa4](https://github.com/gibfahn/up-rs/commit/c607fa44b4d394f46ea153a0ef9c92aa5dbe3f31))
- link: handle multiple link tasks cleaning backup dir in parallel ([4216333](https://github.com/gibfahn/up-rs/commit/4216333c907448977fd344eb10ec4cc7f37582d5))
- tasks: require commas to separate multiple --task values ([6f63932](https://github.com/gibfahn/up-rs/commit/6f639329d570234c4c364e1279418ac53e64d292))

### Features
- run: add a --keep-going option for ignoring bootstrapping errors ([dfb46a0](https://github.com/gibfahn/up-rs/commit/dfb46a00fc94b06ef0afe73f1fd6ded56722cc40))

### Refactor
- log: use tracing macros instead of log macros ([c204f5f](https://github.com/gibfahn/up-rs/commit/c204f5f30a9d362150c641b5b4507dcc2cf835a6))
- opts: switch from slog to tracing for logging ([777d216](https://github.com/gibfahn/up-rs/commit/777d216bf39595a0e4f88dd9ddfa164699729d33))
- rewrite .with_context() to .wrap_err_with() ([ab10034](https://github.com/gibfahn/up-rs/commit/ab100347fca07dfbc6dd9b83515ee2295c7b4308))

<a name="0.13.3"></a>
## [0.13.3][] (2022-01-28)


#### Bug Fixes

* **generate:**  skip git update if nothing changed ([7129a033](https://github.com/gibfahn/up-rs/commit/7129a0331a8e96c3e8907024ca7cc99c198c26e2))
* **task:**  add a suggestion when the task to execute isn't executable ([75e00a28](https://github.com/gibfahn/up-rs/commit/75e00a288f481d2fbd82803f20822447794ae023))



<a name="0.13.2"></a>
## [0.13.2][] (2021-12-02)


#### Bug Fixes

* **opts:**  show full help when running `liv help <subcommand>` ([a7cc667f](https://github.com/gibfahn/up-rs/commit/a7cc667fc9f4c89b20cc2c13ab03759dfa76af96))
* **run:**  allow passing -t for --tasks ([8277d207](https://github.com/gibfahn/up-rs/commit/8277d2072017f0d2075d3e8360bfeb48187c7907))



<a name="0.13.1"></a>
## [0.13.1][] (2021-11-20)


#### Bug Fixes

* **generate:**  replace home dir in generated paths ([50ae6bed](https://github.com/gibfahn/up-rs/commit/50ae6beda3816e7326ace738b0667a3f46681d6a))
* **git:**
  *  update the git remote fetch URL if it has changed ([310f5bb9](https://github.com/gibfahn/up-rs/commit/310f5bb9f4dff24e0ba354a590753ad113793673))
  *  don't try to prune or check unpushed branches on first clone ([65ad74e9](https://github.com/gibfahn/up-rs/commit/65ad74e93a26db4f384103285525a6b32c7e898f))
* **opts:**  allow passing --bootstrap as a long argument ([e0f58ff7](https://github.com/gibfahn/up-rs/commit/e0f58ff74708263532288b33b077fd5c3caaa7a5))
* **run:**  add long options for fallback URL and path ([61bb3ad0](https://github.com/gibfahn/up-rs/commit/61bb3ad0958e20e1780019566091da8a8bc14c3b))
* **self_update:**  don't downgrade pre-release versions ([91d6fd3e](https://github.com/gibfahn/up-rs/commit/91d6fd3e0f25eae7f1294df04f112e00e426558b))
* **tasks:**
  *  include bootstrap tasks in completed task counts ([b746a0b0](https://github.com/gibfahn/up-rs/commit/b746a0b0a2ab81198be3449ccf6575d7af3a78d6))
  *  don't run sudo if running as root already ([3baf0e7e](https://github.com/gibfahn/up-rs/commit/3baf0e7eb79267ecfda68b38cfe497fa2b01bcc7))

#### Features

* **config:**  support org/repo fallback URLs (maps to github.com URLs) ([271785d7](https://github.com/gibfahn/up-rs/commit/271785d7509ca7dd39f048cb550f75ad786fb9fc))



<a name="0.13.0"></a>
## [0.13.0][] (2021-11-15)


#### Features

* **task:**  allow marking a task as skipped in the run_cmd ([81621db1](https://github.com/gibfahn/up-rs/commit/81621db1aa746f8b594f47044343db7c07de4607))

#### Bug Fixes

* **completions:**  have `up completions --help` list available shells ([77bc7dcc](https://github.com/gibfahn/up-rs/commit/77bc7dcc35981837f2f30708ecd15086f9b3c80e))
* **git:**  cannot locate remote-tracking branch 'up/forkmain' ([d9fbe6fd](https://github.com/gibfahn/up-rs/commit/d9fbe6fde2657c67f76a48a029bcceaf731259be))



<a name="0.12.4"></a>
## [0.12.4][] (2021-11-02)


#### Bug Fixes

* **bootstrap:**  fix bootstrapping order ([6b43b100](https://github.com/gibfahn/up-rs/commit/6b43b1009e178084ade10e637ab7438c61b5c868))
* **generate:**
  *  don't recurse into git repos ([cb6346e1](https://github.com/gibfahn/up-rs/commit/cb6346e15e61c1e244266019422cf2e0ca8711a0))
  *  don't serialise empty options in the config ([cd7d52a1](https://github.com/gibfahn/up-rs/commit/cd7d52a1243ac2b4ac65f09343a74f7500829d95))



<a name="0.12.3"></a>
## [0.12.3][] (2021-10-28)


#### Bug Fixes

* **tasks:**
  *  automatically delete broken symlinks to removed tasks ([0d4dfdb3](https://github.com/gibfahn/up-rs/commit/0d4dfdb379a402bec7239c2744e34984bb374f85))
  *  fix command type in stdout/stderr logging ([adf80529](https://github.com/gibfahn/up-rs/commit/adf805299e847baea9a2817f41c58078418f45d5))



<a name="0.12.2"></a>
## [0.12.2][] (2021-10-25)


#### Bug Fixes

* **task:**  rename to run_if_cmd, allow using with run_lib ([8ab63c23](https://github.com/gibfahn/up-rs/commit/8ab63c232c3957869efd31fd5055b0a0d8f56aca))



<a name="0.12.1"></a>
## [0.12.1][] (2021-10-24)




<a name="0.12.0"></a>
## [0.12.0][] (2021-10-24)


#### Bug Fixes

* **defaults:**
  *  skip defaults and its tests on non-Darwin systems ([5c2d0d7a](https://github.com/gibfahn/up-rs/commit/5c2d0d7a24c891219e03b427a88b7134b2656391))
  *  check for container/sandbox app preferences ([71e71676](https://github.com/gibfahn/up-rs/commit/71e716768475fbccace8410d9790b131358d90b5))
  *  continue on defaults errors ([122bdb19](https://github.com/gibfahn/up-rs/commit/122bdb1963c128e96c668a4ed837e68c7cee380d))
* **logging:**
  *  further reduce always run info logging ([296c96b7](https://github.com/gibfahn/up-rs/commit/296c96b79d0c95cb2138a66a0d0b558269aea322))
  *  change initial info log level to debug ([74e8691c](https://github.com/gibfahn/up-rs/commit/74e8691c5f57cd29bd1e110be845dfae465efb9a))
* **tasks:**  return TaskStatus::Skipped if we didn't do any work ([ae8d2a2d](https://github.com/gibfahn/up-rs/commit/ae8d2a2d731f26f5bf3396aaec97c388786c5685))

#### Features

* **defaults:**
  *  support maps and arrays in defaults, restructure up dir ([217d055b](https://github.com/gibfahn/up-rs/commit/217d055bf3e317775e935f786cc201f930bcd998))
  *  add a defaults print command ([3222a369](https://github.com/gibfahn/up-rs/commit/3222a36977f1ff337a2f1ac5c5639bd59d440dfc))



<a name="0.11.0"></a>
## [0.11.0][] (2021-10-18)


#### Bug Fixes

* **git:**  wait longer between retries with fetch failures ([288fadca](https://github.com/gibfahn/up-rs/commit/288fadcaacb023f63b7070fe4d9f78dd0a7eabfc))
* **tasks:**  nicer error messages on task failure ([7b8eee2e](https://github.com/gibfahn/up-rs/commit/7b8eee2e9536a701680a37918e5d54baf20cc61b))



<a name="0.10.0"></a>
## [0.10.0][] (2021-10-04)


#### Features

* **list:**  add an `up list` command ([510e83a7](https://github.com/gibfahn/up-rs/commit/510e83a77001ab592d5ac8e351bc5195b479d39a))

#### Bug Fixes

* **completions:**
  *  better shell completions using ValueHint ([2536528c](https://github.com/gibfahn/up-rs/commit/2536528c31d12c63e92e2e52a55e0a6fb7c8d7eb))
  *  use up as binary name not up-rs ([9097807c](https://github.com/gibfahn/up-rs/commit/9097807c86251ee9829c71608e8b41cd2c2709b3))



<a name="0.9.5"></a>
## [0.9.5][] (2021-09-24)


#### Bug Fixes

* **log:**  don't log a full backtrace on error ([72949e41](https://github.com/gibfahn/up-rs/commit/72949e4116a2187e69c14b647ce0786eba033541))



<a name="0.9.4"></a>
## [0.9.4][] (2021-09-20)


#### Bug Fixes

* **version:**  work around clap 3 beta issue where version isn't guessed ([cc4694fa](https://github.com/gibfahn/up-rs/commit/cc4694fa62c5ef2522589886399e2d29433167de))



<a name="0.9.3"></a>
## [0.9.3][] (2021-09-20)


#### Features

* **completions:**  add command to write completions to stdout ([2dfbc4ef](https://github.com/gibfahn/up-rs/commit/2dfbc4efddedb548baa12006027bed8b4b074a1f))



<a name="0.9.2"></a>
## [0.9.2][] (2021-09-07)


#### Bug Fixes

* **git:**  recursively fetch submodules if we need to check them out ([f152965b](https://github.com/gibfahn/up-rs/commit/f152965b6ecaf11255ccb7ad706d23726bfaea16))



<a name="0.9.1"></a>
## [0.9.1][] (2021-05-17)


#### Bug Fixes

* **generate:**  make task data come last for toml serialization ([f95f5beb](https://github.com/gibfahn/up-rs/commit/f95f5bebc5ac13343dc7b2d0eeed66f1ddd308b2))
* **task:**
  *  make task data optional again ([97ce5641](https://github.com/gibfahn/up-rs/commit/97ce56414aab344b555d9895e0a1196dfebeae38))
  *  have task output log their command type ([e303c232](https://github.com/gibfahn/up-rs/commit/e303c23250e62372703cae74743c1fd2b7657013))

#### Features

* **git:**  warn for git updates that take more than 60s ([d8d38e1b](https://github.com/gibfahn/up-rs/commit/d8d38e1b2940608cd7c626ee5f4f7e40575a47df))



<a name="0.9.0"></a>
## [0.9.0][] (2021-05-17)


#### Performance

* **tasks:**  run all tasks in parallel using Rayon ([4da6e955](https://github.com/gibfahn/up-rs/commit/4da6e955475997e2865df0abfde32c9c3805dc5f))

#### Bug Fixes

* **args:**  support long option name for up --config ([5e21ef28](https://github.com/gibfahn/up-rs/commit/5e21ef28f0c4c817a40f5421ab837a7bf245a130))



<a name="0.8.5"></a>
## [0.8.5][] (2021-03-08)


#### Bug Fixes

* **git:**  handle out-of-date submodules when updating repos ([df8bb072](https://github.com/gibfahn/up-rs/commit/df8bb07263c4a8902e6bb25a080b3ee636dae014))



<a name="0.8.4"></a>
## [0.8.4][] (2021-03-03)


#### Bug Fixes

* **defaults:**  don't write quoted strings as defaults ([079b2d19](https://github.com/gibfahn/up-rs/commit/079b2d19762d37e541dea9f1eba97a056170b84a))
* **git:**
  *  make it easier to copy unmerged fork branches ([424d2209](https://github.com/gibfahn/up-rs/commit/424d2209eb3ac198a001adbc14fdb53f22dd1205))
  *  handle remote.pushDefault that's a URL not remote name ([6ecc38e2](https://github.com/gibfahn/up-rs/commit/6ecc38e249e316a4116b76f3939fead98627e03e))



<a name="0.8.3"></a>
## [0.8.3][] (2021-03-01)


#### Bug Fixes

* **update_self:**  manually implement default for UpdateSelfOptions ([a5ad0378](https://github.com/gibfahn/up-rs/commit/a5ad03783571c9fa44452e27eaa34d418f1cf820))



<a name="0.8.2"></a>
## [0.8.2][] (2021-02-27)


#### Performance

* **git:**  remove double connection to server ([c6e365f4](https://github.com/gibfahn/up-rs/commit/c6e365f4633ae73a08438d9d2fad7e3670780fb8))

#### Bug Fixes

* **self_update:**  allow adding as a task, skip if a dev build ([23bc13c4](https://github.com/gibfahn/up-rs/commit/23bc13c432ca04edc2798661114974c2f5591926))
* **tasks:**  skip broken symlinks in tasks directory ([b90a599c](https://github.com/gibfahn/up-rs/commit/b90a599c71a2096d99d5b0e8285dc3a200b51056))



<a name="0.8.1"></a>
## [0.8.1][] (2021-02-27)


#### Features

* **generate:**  allow providing a sort order for git config generation ([11cc73ba](https://github.com/gibfahn/up-rs/commit/11cc73ba28ea8ea0684cfc94809dcac85cd7e841))
* **update_self:**  allow self update to be called as a lib ([999b9122](https://github.com/gibfahn/up-rs/commit/999b9122f11d728ecf3d4d72521e3293c9bb5132))

#### Bug Fixes

* **git:**  check local git config as well as global ([44b3ebe7](https://github.com/gibfahn/up-rs/commit/44b3ebe7101f7b95b40db806990f31c7622ad646))



<a name="0.8.0"></a>
## [0.8.0][] (2021-02-22)


#### Features

* **defaults:**  add a library to set defaults ([be4bce1b](https://github.com/gibfahn/up-rs/commit/be4bce1b2795e5274d6126929f4db3fe5a6f0c3c))



<a name="0.7.0"></a>
## [0.7.0][] (2021-02-17)


#### Bug Fixes

* **git:**  handle initial repo setup case when checking out branch ([ce657976](https://github.com/gibfahn/up-rs/commit/ce6579764ed8568d80c63778f40131c889ad87d2))
* **link:**  show backup directory path in error message ([b0276613](https://github.com/gibfahn/up-rs/commit/b0276613c66aec325426778638e097436c55f122))

#### Features

* **git:**  warn for unpushed changes ([185209a2](https://github.com/gibfahn/up-rs/commit/185209a23054387a289ea0b9f66afeda140ed976))



<a name="0.6.4"></a>
## [0.6.4][] (2021-02-01)


#### Performance

* **self_update:**  check github API for latest release ([40672199](https://github.com/gibfahn/up-rs/commit/40672199c4fde3d9008dc8a17883988b11e6a5b9))



<a name="0.6.3"></a>
## [0.6.3][] (2021-01-31)


#### Bug Fixes

* **git:**  only ensure repo is clean if we're deleting branches ([b0c4f1f8](https://github.com/gibfahn/up-rs/commit/b0c4f1f84ad7b818e7d167eaae98166c26ef89a9))



<a name="0.6.2"></a>
## [0.6.2][] (2021-01-26)


#### Bug Fixes

* **git:**  don't error if repo dirty unless we actually need to update ([eb335977](https://github.com/gibfahn/up-rs/commit/eb335977d061a27795cd461afca6254ca1102137))



<a name="0.6.1"></a>
## [0.6.1][] (2021-01-21)


#### Bug Fixes

* **git:**  make prune option default to false ([cd616955](https://github.com/gibfahn/up-rs/commit/cd616955a6bf76363c76f9c00d51b3f4489aff25))



<a name="0.6.0"></a>
## [0.6.0][] (2021-01-21)


#### Bug Fixes

* **log:**  make logging less noisy ([c43b471b](https://github.com/gibfahn/up-rs/commit/c43b471bf526900ccf8527ff3b443ba7b8b5ea40))

#### Features

* **git:**  git prune and git cherry implementation ([507e7560](https://github.com/gibfahn/up-rs/commit/507e75600bc23345d1b2d4534299feafb985fea2))
* **main:**  set default file log level to debug not trace ([9d19fc51](https://github.com/gibfahn/up-rs/commit/9d19fc511aaed2d76e9d7327acfaeddf5e73978b))



<a name="0.5.4"></a>
## [0.5.4][] (2020-12-05)


#### Bug Fixes

* **git:**  ignore gitignored files, include git status in error ([d1018d3f](https://github.com/gibfahn/up-rs/commit/d1018d3f9e273677519e556ba4cc0ac5be8e8a37))



<a name="0.5.3"></a>
## [0.5.3][] (2020-12-05)


#### Bug Fixes

* **git:**
  *  make branch update fully update working tree ([303630b1](https://github.com/gibfahn/up-rs/commit/303630b1fbb5b35924aa1167e19b8aabac4af509))
  *  note the -K flag in macOS ssh-add to add to keychain ([9247980c](https://github.com/gibfahn/up-rs/commit/9247980c7bb23c7622f8d282b7cb6c6fc0c63ab3))
* **self_update:**  typo in version check for new versions ([12bc6d0e](https://github.com/gibfahn/up-rs/commit/12bc6d0eb7ef3aece90e30240def9a6ec04d545b))



<a name="0.5.2"></a>
## [0.5.2][] (2020-11-06)


#### Bug Fixes

* **update:**  don't fail immediately on git or link errors ([15f59918](https://github.com/gibfahn/up-rs/commit/15f599184086cd51d3638b0a6a5696341e3d3b6b))



<a name="0.5.1"></a>
## [0.5.1][] (2020-10-31)


#### Features

* **git:**  check @{push} before @{upstream} to ensure up-to-date-ness ([5539d50e](https://github.com/gibfahn/up-rs/commit/5539d50e864c73d3d9e056f914c754bbccc5acbf))
* **self_update:**  allow updating self with `up self` ([699b9087](https://github.com/gibfahn/up-rs/commit/699b9087583eaeadecdc3018f1438202aa0b29bd))



<a name="0.5.0"></a>
## [0.5.0][] (2020-10-26)


#### Bug Fixes

* **git:**
  *  better error message for https auth failure ([bc2225d4](https://github.com/gibfahn/up-rs/commit/bc2225d4160c9d728441a2e83874debc73546253))
  *  add more auth for git fetching ([3df06eac](https://github.com/gibfahn/up-rs/commit/3df06eac0b3467bd20ac2763cc3f2552cd095004))

#### Performance

* **git:**  run git updates in parallel ([77b8d37b](https://github.com/gibfahn/up-rs/commit/77b8d37bee5247b02b8faf5004326b2f3ffd4945))

#### Features

* **generate:**
  *  allow running `up generate` to generate configured tasks ([afef08f7](https://github.com/gibfahn/up-rs/commit/afef08f7ab6250e9372415deb90b20a5780539f8))
  *  add an `up generate git` option to generate configs ([969fc757](https://github.com/gibfahn/up-rs/commit/969fc757379edf92d673fa03a1a670e1e121e98b))
* **git:**  add support for git repo initalization and updates ([e06575a7](https://github.com/gibfahn/up-rs/commit/e06575a766d0e9a57fddcf76d344abcc158946dc))



<a name="0.4.1"></a>
## [0.4.1][] (2020-10-21)


#### Bug Fixes

* **logging:**  handle existing broken symlinks for log path link ([2d3ee577](https://github.com/gibfahn/up-rs/commit/2d3ee577d5981b227ff5e7b0538c825ae1116ee3))



<a name="0.4.0"></a>
## [0.4.0][] (2020-07-22)


#### Features

* **update:**
  *  add bootstrap and a bootstrap_tasks options ([5f1c685f](https://github.com/gibfahn/up-rs/commit/5f1c685f0312cfab5f4cd379e5117268b03c238c))
  *  allow inheriting env and referring to existing env vars ([4470854e](https://github.com/gibfahn/up-rs/commit/4470854eada420efb80ff4987db88dee38e7157b))
  *  run caffeinate on macOS to stay awake while update runs ([d3d431fe](https://github.com/gibfahn/up-rs/commit/d3d431feca8e48de0c4a47869f985f4113c12541))



<a name="0.3.3"></a>
## [0.3.3][] (2020-07-03)


#### Bug Fixes

* **update:**
  *  better logging for check and run commands ([e7bb79ea](https://github.com/gibfahn/up-rs/commit/e7bb79eae87fe5ef3a7615426265e9ba13c5ee84))
  *  better error message when we fail to read a task file ([7c6008b4](https://github.com/gibfahn/up-rs/commit/7c6008b4e8383815ce3cb4e1145588109104db48))



<a name="0.3.1"></a>
## [0.3.1][] (2020-04-17)


#### Bug Fixes

* **clippy:**  fix more clippy and compiler warnings ([64024209](https://github.com/gibfahn/up-rs/commit/64024209a9a86f3cb69143056fa8aa6b1379df8d))
* **lint:**  remove clippy-preview ([38deb9ba](https://github.com/gibfahn/up-rs/commit/38deb9baa1e473ea9dbbdebbf9e5e95b2c38f7b3))
* **test:**  allow TODO comments, forbid XXX ([1ea97cec](https://github.com/gibfahn/up-rs/commit/1ea97cec17d1b651fccaa8e58a2156661ee0b5a3))
* **tests:**  make tests all set a temp dir ([e8b56a94](https://github.com/gibfahn/up-rs/commit/e8b56a9464dacaa8e243f8ed687609cfbe810d68))
* **update:**  only log task stdout/stderr if non-empty ([2615531c](https://github.com/gibfahn/up-rs/commit/2615531c54236048ae8b6563dd55e87fcee222c7))

#### Features

* **fallback:**  add a fallback git repo to get the config from ([d7c6dc9c](https://github.com/gibfahn/up-rs/commit/d7c6dc9c23ae6a260cf6d55481908665afa86ed9))
* **git,log:**  add git update, improve logging ([6087e3e3](https://github.com/gibfahn/up-rs/commit/6087e3e33e7db1b243a836ab2e2d4023cef570de))
* **log:**  make log_dir customisable, make clippy ultra-pedantic ([66cbade9](https://github.com/gibfahn/up-rs/commit/66cbade94299bf3387c8a0f0c8c75558f325301c))



<a name="0.3.0"></a>
## [0.3.0][] (2020-04-17)


#### Bug Fixes

* **config:**  make env optional ([2ab6527c](https://github.com/gibfahn/up-rs/commit/2ab6527c372167c57676619863e672f2d79824a3))

#### Features

* **main:**  log total run time ([0cb59624](https://github.com/gibfahn/up-rs/commit/0cb59624226ad7a8245826efcffe7ed4ad874739))
* **update:**
  *  add task and command duration logging ([1c10e984](https://github.com/gibfahn/up-rs/commit/1c10e9848a3dadae2907c4dd15f1b7a383fe5475))
  *  bootstrap my own update system ([b043e8e5](https://github.com/gibfahn/up-rs/commit/b043e8e50a3565e66403a67ff20a663a7eac6812))
  *  add env support to update scripts ([bf920f74](https://github.com/gibfahn/up-rs/commit/bf920f74630adb5cec9717ac965ef488db8f3e4f))



<a name="0.2.2"></a>
## [0.2.2][] (2020-04-17)


#### Features

* **git:**  statically link openssl ([8d809b3c](https://github.com/gibfahn/up-rs/commit/8d809b3c75c9b028bac79c840567c98547d3928b))



<a name="0.2.1"></a>
## [0.2.1][] (2020-04-17)


#### Features

* **link:**  add an option to `git clone` a repo to link from ([0a8a1352](https://github.com/gibfahn/up-rs/commit/0a8a1352c9abeeb91f7f33ce202ad76f01ee3fe6))

#### Bug Fixes

* **build:**  update to non-yanked version of structopt ([9d5318cf](https://github.com/gibfahn/up-rs/commit/9d5318cf1463737ada80e1da893e3c1f51c9e7e8))



<a name="0.2.0"></a>
## [0.2.0][] (2020-04-17)

[0.2.0]: https://github.com/gibfahn/up-rs/releases/tag/0.2.0
[0.2.1]: https://github.com/gibfahn/up-rs/releases/tag/0.2.1
[0.2.2]: https://github.com/gibfahn/up-rs/releases/tag/0.2.2
[0.3.0]: https://github.com/gibfahn/up-rs/releases/tag/0.3.0
[0.3.1]: https://github.com/gibfahn/up-rs/releases/tag/0.3.1
[0.3.3]: https://github.com/gibfahn/up-rs/releases/tag/0.3.3
[0.4.0]: https://github.com/gibfahn/up-rs/releases/tag/0.4.0
[0.4.1]: https://github.com/gibfahn/up-rs/releases/tag/0.4.1
[0.5.0]: https://github.com/gibfahn/up-rs/releases/tag/0.5.0
[0.5.1]: https://github.com/gibfahn/up-rs/releases/tag/0.5.1
[0.5.2]: https://github.com/gibfahn/up-rs/releases/tag/0.5.2
[0.5.3]: https://github.com/gibfahn/up-rs/releases/tag/0.5.3
[0.5.4]: https://github.com/gibfahn/up-rs/releases/tag/0.5.4
[0.6.0]: https://github.com/gibfahn/up-rs/releases/tag/0.6.0
[0.6.1]: https://github.com/gibfahn/up-rs/releases/tag/0.6.1
[0.6.2]: https://github.com/gibfahn/up-rs/releases/tag/0.6.2
[0.6.3]: https://github.com/gibfahn/up-rs/releases/tag/0.6.3
[0.6.4]: https://github.com/gibfahn/up-rs/releases/tag/0.6.4
[0.7.0]: https://github.com/gibfahn/up-rs/releases/tag/0.7.0
[0.8.0]: https://github.com/gibfahn/up-rs/releases/tag/0.8.0
[0.8.1]: https://github.com/gibfahn/up-rs/releases/tag/0.8.1
[0.8.2]: https://github.com/gibfahn/up-rs/releases/tag/0.8.2
[0.8.3]: https://github.com/gibfahn/up-rs/releases/tag/0.8.3
[0.8.4]: https://github.com/gibfahn/up-rs/releases/tag/0.8.4
[0.8.5]: https://github.com/gibfahn/up-rs/releases/tag/0.8.5
[0.9.0]: https://github.com/gibfahn/up-rs/releases/tag/0.9.0
[0.9.1]: https://github.com/gibfahn/up-rs/releases/tag/0.9.1
[0.9.2]: https://github.com/gibfahn/up-rs/releases/tag/0.9.2
[0.9.3]: https://github.com/gibfahn/up-rs/releases/tag/0.9.3
[0.9.4]: https://github.com/gibfahn/up-rs/releases/tag/0.9.4
[0.9.5]: https://github.com/gibfahn/up-rs/releases/tag/0.9.5
[0.10.0]: https://github.com/gibfahn/up-rs/releases/tag/0.10.0
[0.11.0]: https://github.com/gibfahn/up-rs/releases/tag/0.11.0
[0.12.0]: https://github.com/gibfahn/up-rs/releases/tag/0.12.0
[0.12.1]: https://github.com/gibfahn/up-rs/releases/tag/0.12.1
[0.12.2]: https://github.com/gibfahn/up-rs/releases/tag/0.12.2
[0.12.3]: https://github.com/gibfahn/up-rs/releases/tag/0.12.3
[0.12.4]: https://github.com/gibfahn/up-rs/releases/tag/0.12.4
[0.13.0]: https://github.com/gibfahn/up-rs/releases/tag/0.13.0
[0.13.1]: https://github.com/gibfahn/up-rs/releases/tag/0.13.1
[0.13.2]: https://github.com/gibfahn/up-rs/releases/tag/0.13.2
[0.13.3]: https://github.com/gibfahn/up-rs/releases/tag/0.13.3

<!-- generated by git-cliff -->
