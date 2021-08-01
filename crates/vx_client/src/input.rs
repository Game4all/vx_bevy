use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ElementState},
    prelude::*,
    utils::HashMap,
};
use serde::{Deserialize, Serialize};
use vx_core::{
    platform::UserData,
    utils::{self, Configuration},
};

use std::ops::{Deref, DerefMut};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Action {
    //movement handling
    WalkForward,
    WalkBackward,
    WalkRight,
    WalkLeft,
    WalkJump,
    WalkCrouch,
    CursorLock,

    //debug stuff
    ToggleDebugUi,
    RemeshChunks,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum KeyButton {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

//todo: this is a super simple action map but it may be cool to move to something like **Kurinji** when it updates to bevy 0.5
#[derive(Serialize, Deserialize)]
pub struct Keybindings(HashMap<KeyButton, Action>);

impl Deref for Keybindings {
    type Target = HashMap<KeyButton, Action>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Keybindings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Configuration for Keybindings {
    const FILENAME: &'static str = "bindings.ron";
}

impl Default for Keybindings {
    fn default() -> Self {
        let mut keybinds = HashMap::default();
        keybinds.insert(KeyButton::Keyboard(KeyCode::Z), Action::WalkForward);
        keybinds.insert(KeyButton::Keyboard(KeyCode::S), Action::WalkBackward);
        keybinds.insert(KeyButton::Keyboard(KeyCode::A), Action::WalkLeft);
        keybinds.insert(KeyButton::Keyboard(KeyCode::D), Action::WalkRight);
        keybinds.insert(KeyButton::Keyboard(KeyCode::Space), Action::WalkJump);
        keybinds.insert(KeyButton::Keyboard(KeyCode::LShift), Action::WalkCrouch);
        keybinds.insert(KeyButton::Keyboard(KeyCode::Escape), Action::CursorLock);
        keybinds.insert(KeyButton::Keyboard(KeyCode::F3), Action::ToggleDebugUi);
        keybinds.insert(KeyButton::Keyboard(KeyCode::F5), Action::RemeshChunks);
        Keybindings(keybinds)
    }
}

fn update_actions(
    mut actions: ResMut<Input<Action>>,
    keybinds: Res<Keybindings>,
    mut kb_events: EventReader<KeyboardInput>,
    mut mouse_events: EventReader<MouseButtonInput>,
) {
    actions.clear();
    for kb_event in kb_events.iter() {
        if let KeyboardInput {
            key_code: Some(key_code),
            state,
            ..
        } = kb_event
        {
            if let Some(action) = keybinds.get(&KeyButton::Keyboard(*key_code)) {
                match state {
                    ElementState::Pressed => actions.press(*action),
                    ElementState::Released => actions.release(*action),
                }
            }
        }
    }

    for mouse_event in mouse_events.iter() {
        let MouseButtonInput { button, state } = mouse_event;
        if let Some(action) = keybinds.get(&KeyButton::Mouse(*button)) {
            match state {
                ElementState::Pressed => actions.press(*action),
                ElementState::Released => actions.release(*action),
            }
        }
    }
}

fn load_bindings(In(bindings): In<Option<Keybindings>>, mut user_bindings: ResMut<Keybindings>) {
    match bindings {
        Some(keybinds) => {
            for key in keybinds.keys() {
                if user_bindings.contains_key(key) {
                    // unmap a key from the previous binding if an action in the loaded bindings uses that key
                    user_bindings.remove(key);
                }
                user_bindings.insert(*key, *keybinds.get_key_value(key).unwrap().1);
            }
            info!("Bindings loaded successfully");
        }
        None => {}
    }
}

fn save_bindings(
    binds: Res<Keybindings>,
    mut exit_events: EventReader<AppExit>,
    userdata: Res<UserData>,
) {
    for _ in exit_events.iter() {
        utils::save_config_file(userdata, binds);
        break;
    }
}

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Keybindings>()
            .init_resource::<Input<Action>>()
            .add_startup_system(
                utils::load_config_file::<Keybindings>
                    .system()
                    .chain(load_bindings.system()),
            )
            .add_system(update_actions.system())
            .add_system(save_bindings.system());
    }
}
