use crate::common::{
    paths::BLOCK0_BIN,
    startup::{
        empty_db,
        server::{dump_settings, BootstrapCommandBuilder, ServerSettingsBuilder},
    },
};
use assert_cmd::assert::OutputAssertExt;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
    str::FromStr,
};

#[test]
pub fn no_in_settings_provided() {
    let command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .build()
        .assert()
        .failure()
        .stdout(predicate::str::contains("Unable to open the database file"));
}

#[test]
pub fn in_settings_file_does_not_exist() {
    let mut command_builder: BootstrapCommandBuilder = Default::default();

    let non_existing_file = PathBuf::from_str("settings.json").unwrap();

    command_builder
        .in_settings_file(&non_existing_file)
        .build()
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Error reading file settings.json: The system cannot find the file specified",
        ));
}

#[test]
pub fn in_settings_file_malformed() {
    let temp_dir = TempDir::new().unwrap();

    let mut settings_builder: ServerSettingsBuilder = Default::default();
    let settings = settings_builder
        .with_random_localhost_address()
        .with_db_path(empty_db(&temp_dir).to_str().unwrap())
        .with_block0_path(BLOCK0_BIN)
        .build();

    let settings_file = dump_settings(&temp_dir, &settings);
    remove_first_char_in_file(&settings_file);

    let mut command_builder: BootstrapCommandBuilder = Default::default();
    command_builder
        .in_settings_file(&settings_file)
        .build()
        .assert()
        .failure()
        .stdout(predicate::str::contains("Error loading settings from file"));
}

pub fn remove_first_char_in_file(settings_file: &PathBuf) {
    let mut file = File::open(settings_file).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    drop(file);
    contents.remove(0);
    fs::write(settings_file, contents).expect("Unable to write file");
}

#[test]
pub fn in_settings_file_with_malformed_path() {
    let mut command_builder: BootstrapCommandBuilder = Default::default();

    let non_existing_file = PathBuf::from_str("/tmp/a/c:/settings.json").unwrap();

    command_builder
        .in_settings_file(&non_existing_file)
        .build()
        .assert()
        .failure()
        .stderr(predicate::str::contains("label syntax is incorrect"));
}
