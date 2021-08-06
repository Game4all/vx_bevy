use crate::platform::UserData;
use crate::utils::{self, Configuration};
use bevy::app::AppExit;
use bevy::log::info;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GlobalConfig {
    pub render_distance: i32,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            render_distance: 8,
        }
    }
}

fn load_config(In(config): In<Option<GlobalConfig>>, mut cfg: ResMut<GlobalConfig>) {
    if let Some(loaded_cfg) = config {
        *cfg = loaded_cfg;
        info!("Loaded global config.");
    }
}

impl Configuration for GlobalConfig {
    const FILENAME: &'static str = "config.ron";
}

fn save_config(
    binds: Res<GlobalConfig>,
    mut exit_events: EventReader<AppExit>,
    userdata: Res<UserData>,
) {
    for _ in exit_events.iter() {
        utils::save_config_file(userdata, binds);
        break;
    }
}

pub struct ConfigurationPlugin;

impl Plugin for ConfigurationPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.init_resource::<GlobalConfig>()
            .add_startup_system(
                utils::load_config_file::<GlobalConfig>
                    .system()
                    .chain(load_config.system()),
            )
            .add_system(save_config.system());
    }
}
