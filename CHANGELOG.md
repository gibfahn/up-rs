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
