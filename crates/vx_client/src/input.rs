use bevy::{
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
    utils::HashMap,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
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

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource::<KeybindingMap>(default_keybinds())
            .init_resource::<Input<Action>>()
            .add_system(update_actions.system());
    }
}
