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
