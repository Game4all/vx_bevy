use crate::platform::UserData;
use bevy::{
    app::AppExit,
    ecs::component::Component,
    prelude::*,
    utils::tracing::{error, info},
};
use ron::ser::PrettyConfig;
use serde::{de::DeserializeOwned, Serialize};

use std::{any::type_name, fs::OpenOptions, io::Write, marker::PhantomData, path::PathBuf};

/// A trait for marking structs as configuration types which can be loaded / saved to disk.
pub trait Configuration: DeserializeOwned + Serialize {
    const FILENAME: &'static str;
}

/// Generic system to read a config type from disk.
fn load_config_file<T: Component + Configuration>(userdata: Res<UserData>) -> Option<T> {
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

fn load_config<T: Component + Configuration>(In(config): In<Option<T>>, mut cfg: ResMut<T>) {
    if let Some(usable_config) = config {
        *cfg = usable_config;
    }

    info!("Loaded {} successfully.", type_name::<T>());
}

fn save_config_file<T: Component + Configuration>(
    userdata: Res<UserData>,
    config: Res<T>,
    mut quit_events: EventReader<AppExit>,
) {
    for _ in quit_events.iter() {
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

        info!("{} was saved.", type_name::<T>());
        break;
    }
}

#[derive(Default)]
pub struct ConfigurationPlugin<T: Component + Configuration + Default>(PhantomData<T>);

impl<T: Component + Configuration + Default> Plugin for ConfigurationPlugin<T> {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<T>()
            .add_startup_system(
                load_config_file::<T>
                    .system()
                    .chain(load_config::<T>.system()),
            )
            .add_system(save_config_file::<T>.system());
    }
}
