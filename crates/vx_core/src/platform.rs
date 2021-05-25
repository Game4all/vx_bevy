use std::{
    fs,
    io::Result,
    path::{Path, PathBuf},
};

use bevy::prelude::*;

#[derive(Debug)]
pub struct UserData {
    pub root: PathBuf,
}

impl UserData {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn dir_exists<T: AsRef<Path>>(&self, path: &T) -> bool {
        let mut full_path = self.root();
        full_path.push(path);
        full_path.is_dir()
    }

    pub fn create_dir<T: AsRef<Path>>(&self, path: &T) -> Result<()> {
        let mut full_path = self.root();
        full_path.push(path);
        fs::create_dir_all(&full_path)
    }

    pub fn root(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn absolute_path<T: AsRef<Path>>(&self, path: T) -> PathBuf {
        let mut abs_path = self.root();
        abs_path.push(&path);
        abs_path
    }
}

fn setup_userdata(userdata: ResMut<UserData>) {
    userdata.create_dir(&PathBuf::from(".")).unwrap();
}
pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let mut user_data_dir = std::env::current_dir().unwrap();
        user_data_dir.push("userdata");

        app.insert_resource(UserData::new(user_data_dir))
            .add_startup_system(setup_userdata.system());
    }
}
