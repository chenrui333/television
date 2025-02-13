use std::collections::HashMap;

use color_eyre::Result;
use television_channels::cable::{CableChannelPrototype, CableChannels};
use tracing::debug;

use crate::config::get_config_dir;

/// Just a proxy struct to deserialize prototypes
#[derive(Debug, serde::Deserialize, Default)]
struct ChannelPrototypes {
    #[serde(rename = "cable_channel")]
    prototypes: Vec<CableChannelPrototype>,
}

const CABLE_FILE_NAME_SUFFIX: &str = "channels";
const CABLE_FILE_FORMAT: &str = "toml";

#[cfg(unix)]
const DEFAULT_CABLE_CHANNELS: &str =
    include_str!("../../cable/unix-channels.toml");

#[cfg(not(unix))]
const DEFAULT_CABLE_CHANNELS: &str =
    include_str!("../../cable/windows-channels.toml");

/// Load the cable configuration from the config directory.
///
/// Cable is loaded by compiling all files that match the following
/// pattern in the config directory: `*channels.toml`.
///
/// # Example:
/// ```
///   config_folder/
///   ├── cable_channels.toml
///   ├── my_channels.toml
///   └── windows_channels.toml
/// ```
pub fn load_cable_channels() -> Result<CableChannels> {
    let config_dir = get_config_dir();

    // list all files in the config directory
    let files = std::fs::read_dir(&config_dir)?;

    // filter the files that match the pattern
    let file_paths = files
        .filter_map(|f| f.ok().map(|f| f.path()))
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map_or(false, |e| e.to_lowercase() == CABLE_FILE_FORMAT)
        })
        .filter(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .map_or(false, |s| s.ends_with(CABLE_FILE_NAME_SUFFIX))
        });

    let user_defined_prototypes = file_paths.fold(Vec::new(), |mut acc, p| {
        let r: ChannelPrototypes = toml::from_str(
            &std::fs::read_to_string(p)
                .expect("Unable to read configuration file"),
        )
        .unwrap_or_default();
        acc.extend(r.prototypes);
        acc
    });

    debug!("Loaded cable channels: {:?}", user_defined_prototypes);

    let default_prototypes: ChannelPrototypes =
        toml::from_str(DEFAULT_CABLE_CHANNELS)
            .expect("Unable to parse default cable channels");

    let mut cable_channels = HashMap::new();
    // chaining default with user defined prototypes so that users may override the
    // default prototypes
    for prototype in default_prototypes
        .prototypes
        .into_iter()
        .chain(user_defined_prototypes)
    {
        cable_channels.insert(prototype.name.clone(), prototype);
    }
    Ok(CableChannels(cable_channels))
}
