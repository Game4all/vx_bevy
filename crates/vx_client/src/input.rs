use bevy::{
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
    utils::tracing::{error, info},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};
use vx_core::platform::UserData;

use std::{fs::OpenOptions, path::PathBuf};

const BINDINGS_FILENAME: &'static str = "bindings.ron";

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Action {
    //movement handling
    WalkForward,
    WalkBackward,
    WalkRight,
    WalkLeft,
    CursorLock,
}

//todo: this is a super simple action map but it may be cool to move to something like **Kurinji** when it updates to bevy 0.5
pub type KeybindingMap = HashMap<KeyCode, Action>;

fn default_keybinds() -> KeybindingMap {
    let mut keybinds = KeybindingMap::default();
    keybinds.insert(KeyCode::Z, Action::WalkForward);
    keybinds.insert(KeyCode::S, Action::WalkBackward);
    keybinds.insert(KeyCode::A, Action::WalkLeft);
    keybinds.insert(KeyCode::D, Action::WalkRight);
    keybinds.insert(KeyCode::Escape, Action::CursorLock);
    keybinds
}

fn update_actions(
    mut actions: ResMut<Input<Action>>,
    keybinds: Res<KeybindingMap>,
    mut key_events: EventReader<KeyboardInput>,
) {
    actions.clear();
    for event in key_events.iter() {
        if let KeyboardInput {
            key_code: Some(key_code),
            state,
            ..
        } = event
        {
            if let Some(action) = keybinds.get(key_code) {
                match state {
                    ElementState::Pressed => actions.press(*action),
                    ElementState::Released => actions.release(*action),
                }
            }
        }
    }
}

fn load_bindings(userdata: Res<UserData>, mut user_bindings: ResMut<KeybindingMap>) {
    info!("Loading user bindings");

    let file = match userdata.open(
        &PathBuf::from(BINDINGS_FILENAME),
        OpenOptions::new().read(true),
    ) {
        Ok(file) => file,
        Err(err) => {
            error!("Failed to load user bindings: {:?}", err);
            return;
        }
    };

    let loaded_bindings: KeybindingMap = match ron::de::from_reader(file) {
        Ok(data) => data,
        Err(error) => {
            error!("Failed to parse user bindings: {:?}", error);
            return;
        }
    };

    for key in loaded_bindings.keys() {
        if user_bindings.contains_key(key) {
            // unmap a key from the previous binding if an action in the loaded bindings uses that key
            user_bindings.remove(key);
        }
        user_bindings.insert(*key, *loaded_bindings.get_key_value(key).unwrap().1);
    }

    info!("User bindings loaded!");
}

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource::<KeybindingMap>(default_keybinds())
            .init_resource::<Input<Action>>()
            .add_system(update_actions.system())
            .add_startup_system(load_bindings.system());
    }
}
