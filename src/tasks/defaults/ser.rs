//! Helper to allow serializing plists containing binary data to yaml.

use color_eyre::Result;
use plist::Value;
use std::mem;

/// Replace binary data attributes to work around <https://github.com/dtolnay/serde-yaml/issues/91>.
pub(super) fn replace_data_in_plist(value: &mut Value) -> Result<()> {
    let mut stringified_data_value = match value {
        Value::Array(arr) => {
            for el in arr.iter_mut() {
                replace_data_in_plist(el)?;
            }
            return Ok(());
        }
        Value::Dictionary(dict) => {
            for (_, v) in dict.iter_mut() {
                replace_data_in_plist(v)?;
            }
            return Ok(());
        }
        Value::Data(bytes) => Value::String(hex::encode(bytes)),
        _ => {
            return Ok(());
        }
    };
    mem::swap(value, &mut stringified_data_value);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::tasks::defaults::ser::replace_data_in_plist;
    use color_eyre::Result;
    use test_log::test;
    use testutils::ensure_eq;
    use tracing::info;

    #[test]
    fn test_serialize_binary() -> Result<()> {
        // Modified version of ~/Library/Preferences/com.apple.humanunderstanding.plist
        let binary_plist_as_hex = "62706c6973743030d101025f10124861736847656e657261746f722e73616c744f10201111111122222222333333334444444455555555666666667777777788888888080b200000000000000101000000000000000300000000000000000000000000000043";
        let expected_yaml = "HashGenerator.salt: \
                             '1111111122222222333333334444444455555555666666667777777788888888'\n";

        let binary_plist = hex::decode(binary_plist_as_hex)?;

        let mut value: plist::Value = plist::from_bytes(&binary_plist)?;
        info!("Value before: {value:?}");
        replace_data_in_plist(&mut value)?;
        info!("Value after: {value:?}");
        let yaml_string = serde_yaml::to_string(&value)?;
        info!("Yaml value: {yaml_string}");
        ensure_eq!(expected_yaml, yaml_string);
        Ok(())
    }
}
