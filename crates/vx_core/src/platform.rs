use std::{
    fs::{self, ReadDir},
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

    /// Enumerates all the entries at the provided path.
    pub fn enum_dir<T: AsRef<Path>>(&self, path: &T) -> Result<ReadDir> {
        let mut full_path = self.root();
        full_path.push(path);
        fs::read_dir(path)
    }

    /// Creates a directory at the specified path.
    pub fn create_dir<T: AsRef<Path>>(&self, path: &T) -> Result<()> {
        let mut full_path = self.root();
        full_path.push(path);
        fs::create_dir_all(&full_path)
    }

    /// Checks for existence of the given path.
    pub fn exists<T: AsRef<Path>>(&self, path: &T) -> bool {
        let mut full_path = self.root();
        full_path.push(path);
        full_path.exists()
    }

    pub fn root(&self) -> PathBuf {
        self.root.clone()
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
