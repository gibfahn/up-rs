//! Utilities for dealing with users.

use crate::cmd_debug;
use crate::exec::UpDuct;
use color_eyre::Result;
use duct::Expression;
use std::thread;
use std::time::Duration;
use tracing::debug;
use tracing::info;
use tracing::trace;
use tracing::warn;

/**
Prompt user for sudo if necessary, then keep running sudo in the background to keep access till we exit.

There are three common cases:

| State                   | `sudo -n true` | `sudo -kn true` |
| ---                     | ---            | ---             |
| Normal Mac              | ❌             | ❌              |
| Normal Mac, cached sudo | ✅             | ❌              |
| Passwordless sudo       | ✅             | ✅              |

Example of a modified machine that has passwordless sudo enabled for the current user:

```console
local@Mac-mini-5 ~ % sudo -l
User local may run the following commands on Mac-mini-5:
    (ALL) ALL
    (ALL) NOPASSWD: ALL
```

*/
pub(crate) fn get_and_keep_sudo(yes: bool) -> Result<()> {
    debug!(
        "Current tty is: {}",
        cmd_debug!("tty")
            .read()
            .unwrap_or_else(|e| format!("Failed with: {e}"))
    );
    if current_user_is_root() {
        debug!("Not getting sudo as we're already running as root.");
        return Ok(());
    }

    // Run `sudo -n true` && `sudo -kn true`:
    // - normal mac, no sudo: fail, fail -> run sudo -v
    // - normal mac, with sudo cached creds: pass, fail -> run sudo -v
    // - devicecompute mac: pass, pass -> do nothing
    if cmd_debug!("sudo", "-kn", "true")
        .stderr_null()
        .run_with(Expression::stdout_null)
        .is_ok()
    {
        info!("Looks like passwordless sudo is enabled, not prompting for sudo.");
        return Ok(());
    }

    // If `--yes` flag set, use `sudo -n` so we don't prompt for password input.
    let sudo_arg = if yes { "-vn" } else { "-v" };

    info!("Prompting for your sudo password (the one you use to log in to this Mac)...");

    cmd_debug!("sudo", sudo_arg).run_with(Expression::stdout_to_stderr)?;
    thread::spawn(|| {
        // Only refresh sudo for max 24 hours.
        for _ in 1..1440 {
            thread::sleep(Duration::from_secs(60));
            if let Err(e) = cmd_debug!("sudo", "-vn").run_with(Expression::stdout_to_stderr) {
                warn!("Refreshing sudo with 'sudo -vn' failed with: {e:#}");
            }
        }
    });
    Ok(())
}

/// Return whether we are running as root.
pub(crate) fn current_user_is_root() -> bool {
    let current_user_id = uzers::get_current_uid();
    trace!("Found current user ID to be: {current_user_id}");
    current_user_id == 0
}
