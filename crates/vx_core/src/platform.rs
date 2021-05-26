use std::{
    fs::{self, File, OpenOptions, ReadDir},
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
        fs::read_dir(self.full_path(path))
    }

    /// Creates a directory at the specified path.
    pub fn create_dir<T: AsRef<Path>>(&self, path: &T) -> Result<()> {
        fs::create_dir_all(self.full_path(path))
    }

    /// Checks for existence of the given path.
    pub fn exists<T: AsRef<Path>>(&self, path: &T) -> bool {
        self.full_path(path).exists()
    }

    /// Deletes the file at the given path.
    pub fn delete_file<T: AsRef<Path>>(&self, path: &T) -> Result<()> {
        fs::remove_file(self.full_path(path))
    }

    /// Deletes the directory and all its hierarchy at the given path.
    pub fn delete_dir<T: AsRef<Path>>(&self, path: &T) -> Result<()> {
        fs::remove_dir_all(self.full_path(path))
    }

    /// Opens the file at the given path with the specified open options.
    pub fn open<T: AsRef<Path>>(&self, path: &T, open_options: &OpenOptions) -> Result<File> {
        open_options.open(self.full_path(path))
    }

    // Gets the root path of the user data.
    pub fn root(&self) -> PathBuf {
        self.root.clone()
    }

    fn full_path<T: AsRef<Path>>(&self, path: &T) -> PathBuf {
        let mut full_path = self.root();
        full_path.push(path);
        full_path
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
