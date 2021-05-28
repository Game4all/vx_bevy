use crate::platform::UserData;
use bevy::{
    ecs::component::Component,
    prelude::*,
    utils::tracing::{error, info},
};
use ron::ser::PrettyConfig;
use serde::{de::DeserializeOwned, Serialize};

use std::{any::type_name, fs::OpenOptions, io::Write, path::PathBuf};

/// A trait for marking structs as configuration types which can be loaded / saved to disk.
pub trait Configuration: DeserializeOwned + Serialize {
    const FILENAME: &'static str;
}

/// Generic system to read a config type from disk.
pub fn load_config_file<T: Component + Configuration>(userdata: Res<UserData>) -> Option<T> {
    info!("Loading {} from {}", type_name::<T>(), T::FILENAME);

    let file = match userdata.open(&PathBuf::from(T::FILENAME), OpenOptions::new().read(true)) {
        Ok(file) => file,
        Err(err) => {
            error!(
                "Failed to load {} from {}: {:?}",
                type_name::<T>(),
                T::FILENAME,
                err
            );
            return None;
        }
    };

    return match ron::de::from_reader(file) {
        Ok(data) => Some(data),
        Err(error) => {
            error!(
                "Failed to parse {} from {}: {:?}",
                type_name::<T>(),
                T::FILENAME,
                error
            );
            return None;
        }
    };
}

pub fn save_config_file<T: Component + Configuration>(userdata: Res<UserData>, config: Res<T>) {
    info!("Saving {} to {}", type_name::<T>(), T::FILENAME);

    let mut file = match userdata.open(
        &PathBuf::from(T::FILENAME),
        OpenOptions::new().create(true).write(true),
    ) {
        Ok(file) => file,
        Err(err) => {
            error!(
                "Failed to save {} to {}: {:?}",
                type_name::<T>(),
                T::FILENAME,
                err
            );
            return;
        }
    };

    let prettier_config = PrettyConfig::new()
        .with_depth_limit(2)
        .with_separate_tuple_members(true)
        .with_enumerate_arrays(true);

    let serialized_data = ron::ser::to_string_pretty(&*config, prettier_config)
        .expect("Failed to serialize configuration.");
    file.write_all(serialized_data.as_bytes())
        .expect("Failed to write config to disk.");

    info!("Config {} was saved.", type_name::<T>());
}
