use crate::utils::Configuration;
use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Component)]
pub struct GlobalConfig {
    pub render_distance: i32,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig { render_distance: 8 }
    }
}

impl Configuration for GlobalConfig {
    const FILENAME: &'static str = "config.ron";
}
