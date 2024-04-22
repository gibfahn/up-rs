//! macOS specific functions.

use crate::cmd_debug;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use serde::Deserialize;
use serde::Serialize;

/// Get the hardware UUID of the current Mac.
/// You can get the Hardware UUID from:
/// <https://apple.stackexchange.com/questions/342042/how-can-i-query-the-hardware-uuid-of-a-mac-programmatically-from-a-command-line>
pub(crate) fn get_hardware_uuid() -> Result<String> {
    let raw_output = cmd_debug!("ioreg", "-d2", "-a", "-c", "IOPlatformExpertDevice").read()?;
    let ioreg_output: IoregOutput = plist::from_bytes(raw_output.as_bytes())?;
    Ok(ioreg_output
        .io_registry_entry_children
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("Failed to get the Hardware UUID for the current Mac."))?
        .io_platform_uuid)
}

/// XML output returned by `ioreg -d2 -a -c IOPlatformExpertDevice`
#[derive(Debug, Clone, Deserialize, Serialize)]
struct IoregOutput {
    /// The set of `IORegistry` entries.
    #[serde(rename = "IORegistryEntryChildren")]
    io_registry_entry_children: Vec<IoRegistryEntryChildren>,
}

/// A specific `IORegistry` entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct IoRegistryEntryChildren {
    /// The platform UUID.
    #[serde(rename = "IOPlatformUUID")]
    io_platform_uuid: String,
}

#[cfg(target_os = "macos")]
#[cfg(test)]
mod tests {
    use color_eyre::Result;
    use testutils::ensure_eq;

    #[test]
    fn test_get_hardware_uuid() -> Result<()> {
        use crate::cmd;
        use crate::utils::mac::get_hardware_uuid;

        let system_profiler_output = cmd!("system_profiler", "SPHardwareDataType").read()?;
        let expected_value = system_profiler_output
            .lines()
            .find_map(|line| {
                line.contains("UUID")
                    .then(|| line.split_whitespace().last().unwrap())
            })
            .unwrap();
        let actual_value = get_hardware_uuid()?;
        ensure_eq!(expected_value, actual_value);
        Ok(())
    }
}
