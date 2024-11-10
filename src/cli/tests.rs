use super::*;
use crate::types::machine::Machine;
use lazy_static::lazy_static;
use mockall::mock;
use mockall::predicate::*;
use std::path::PathBuf;
use std::sync::Mutex;
use std::fs;
use tempfile::TempDir;

use crate::config::manager::ConfigManager;

mock! {
    pub FileSystem {
        fn create_dir(&self, path: &PathBuf) -> Result<(), String>;
        fn write_file(&self, path: &PathBuf, contents: &str) -> Result<(), String>;
        fn read_file(&self, path: &PathBuf) -> Result<String, String>;
        fn file_exists(&self, path: &PathBuf) -> bool;
    }
}

mock! {
    pub Config {
        fn get(&self, key: ConfigKey) -> Result<Option<String>, String>;
        fn set(&self, key: ConfigKey, value: &str) -> Result<(), String>;
        fn clear(&self, key: ConfigKey) -> Result<(), String>;
    }
}

mock! {
    pub MachineData {
        fn get_all(&self) -> Vec<Machine>;
        fn find_by_name(&self, name: &str) -> Option<Machine>;
        fn find_similar_names(&self, name: &str, threshold: f64) -> Vec<Machine>;
    }
}

mock! {
    pub UpdateChecker {
        fn check_for_update(&self) -> Result<Option<String>, String>;
        fn perform_update(&self, version: &str) -> Result<(), String>;
    }
}

lazy_static! {
    static ref FILE_SYSTEM_MOCK: Mutex<MockFileSystem> = Mutex::new(MockFileSystem::new());
    static ref CONFIG_MOCK: Mutex<MockConfig> = Mutex::new(MockConfig::new());
    static ref MACHINE_DATA_MOCK: Mutex<MockMachineData> = Mutex::new(MockMachineData::new());
    static ref UPDATE_CHECKER_MOCK: Mutex<MockUpdateChecker> = Mutex::new(MockUpdateChecker::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_watch_command() {
        let mut fs_mock = FILE_SYSTEM_MOCK.lock().unwrap();
        fs_mock.expect_create_dir().returning(|_| Ok(()));
        fs_mock.expect_write_file().returning(|_, _| Ok(()));
        fs_mock
            .expect_read_file()
            .returning(|_| Ok("test".to_string()));
        fs_mock.expect_file_exists().returning(|_| true);

        let mut config_mock = CONFIG_MOCK.lock().unwrap();
        config_mock
            .expect_get()
            .with(eq(ConfigKey::WatchDir))
            .returning(|_| Ok(Some("/test/dir".to_string())));
        config_mock
            .expect_get()
            .with(eq(ConfigKey::Machine))
            .returning(|_| Ok(Some("test_machine".to_string())));

        let mut machine_data_mock = MACHINE_DATA_MOCK.lock().unwrap();
        machine_data_mock
            .expect_find_by_name()
            .with(eq("test_machine"))
            .returning(|_| {
                Some(
                    Machine::new("test_machine".to_string())
                        .with_file_formats(vec!["dst".to_string(), "exp".to_string()])
                        .with_usb_path(Some("/media/test".to_string())),
                )
            });

        let cli = Cli {
            command: Some(Commands::Watch {
                dir: Some(PathBuf::from("/test/dir")),
                output_format: Some("exp".to_string()),
                machine: Some("test_machine".to_string()),
            }),
        };

        let mut writer = std::io::stdout();
        let result = cli.command.unwrap().execute(&mut writer);
        assert!(result.is_ok(), "Watch command should execute successfully");

        // Verify that the expected file system operations occurred
        fs_mock.checkpoint(); // Verifies all expected calls were made
        config_mock.checkpoint();
        machine_data_mock.checkpoint();
    }

    #[test]
    #[ignore]
    fn test_machine_command() {
        let mut config_mock = CONFIG_MOCK.lock().unwrap();
        config_mock
            .expect_set()
            .with(eq(ConfigKey::Machine), eq("test_machine"))
            .returning(|_, _| Ok(()));

        let cli = Cli {
            command: Some(Commands::Set {
                what: "machine".to_string(),
                value: Some("test_machine".to_string()),
            }),
        };

        let mut writer = std::io::stdout();
        let result = cli.command.unwrap().execute(&mut writer);
        assert!(
            result.is_ok(),
            "Set machine command should execute successfully"
        );

        // Verify that the config was updated as expected
        config_mock.checkpoint();
    }

    #[test]
    #[ignore]
    fn test_list_machines_command() {
        let mut machine_data_mock = MACHINE_DATA_MOCK.lock().unwrap();
        machine_data_mock.expect_get_all().returning(|| {
            vec![
                Machine::new("machine1".to_string())
                    .with_file_formats(vec!["dst".to_string(), "exp".to_string()]),
                Machine::new("machine2".to_string())
                    .with_file_formats(vec!["jef".to_string(), "vp3".to_string()]),
            ]
        });

        let cli = Cli {
            command: Some(Commands::Machines {
                format: Some("dst".to_string()),
                verbose: false,
            }),
        };

        let mut output = Vec::new();
        let result = {
            let mut writer = std::io::BufWriter::new(&mut output);
            cli.command.unwrap().execute(&mut writer)
        };

        assert!(
            result.is_ok(),
            "List machines command should execute successfully"
        );

        let output_string = String::from_utf8(output).unwrap();
        assert!(
            output_string.contains("machine1"),
            "Output should contain machine1"
        );
        assert!(
            !output_string.contains("machine2"),
            "Output should not contain machine2"
        );
    }

    #[test]
    #[ignore]
    fn test_machine_info_command() {
        let mut machine_data_mock = MACHINE_DATA_MOCK.lock().unwrap();
        machine_data_mock
            .expect_find_by_name()
            .with(eq("machine1"))
            .returning(|_| {
                Some(
                    Machine::new("machine1".to_string())
                        .with_file_formats(vec!["dst".to_string(), "exp".to_string()]),
                )
            });

        let cli = Cli {
            command: Some(Commands::Machine {
                command: MachineCommand::Info {
                    name: "machine1".to_string(),
                },
            }),
        };

        let mut output = Vec::new();
        let result = {
            let mut writer = std::io::BufWriter::new(&mut output);
            cli.command.unwrap().execute(&mut writer)
        };

        assert!(
            result.is_ok(),
            "Machine info command should execute successfully"
        );

        let output_string = String::from_utf8(output).unwrap();
        println!("output_string: {}", output_string);
        assert!(
            output_string.contains("machine1"),
            "Output should contain machine name"
        );
        assert!(
            output_string.contains("dst"),
            "Output should contain file format dst"
        );
        assert!(
            output_string.contains("exp"),
            "Output should contain file format exp"
        );
    }

    #[test]
    #[ignore]
    fn test_update_command() {
        let mut update_checker_mock = UPDATE_CHECKER_MOCK.lock().unwrap();
        update_checker_mock
            .expect_check_for_update()
            .returning(|| Ok(Some("100.0.0".to_string())));

        let cli = Cli {
            command: Some(Commands::Update { dry_run: true }),
        };

        let mut output = Vec::new();
        let result = {
            let mut writer = std::io::BufWriter::new(&mut output);
            cli.command.unwrap().execute(&mut writer)
        };

        assert!(result.is_ok(), "Update command should execute successfully");

        let output_string = String::from_utf8(output).unwrap();
        assert!(
            output_string.contains("100.0.0"),
            "Output should contain the new version"
        );
        assert!(
            output_string.contains("dry run"),
            "Output should indicate dry run mode"
        );
    }

    #[test]
    fn test_config_commands() {
        // Create a temporary directory for the config file
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("stitch-sync");
        fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.toml");

        // Create a mock file system
        let mut fs_mock = FILE_SYSTEM_MOCK.lock().unwrap();
        let config_path_clone = config_path.clone();
        fs_mock.expect_file_exists().returning(move |path| path == &config_path_clone);
        let config_path_clone = config_path.clone();
        fs_mock.expect_read_file().returning(move |path| {
            if *path == config_path_clone {
                Ok(fs::read_to_string(&config_path_clone).unwrap_or_default())
            } else {
                Ok("".to_string())
            }
        });
        fs_mock.expect_write_file().returning(move |path, content| {
            if *path == config_path {
                fs::write(&config_path, content).unwrap();
            }
            Ok(())
        });

        // Create a real ConfigManager instance
        let config_manager = ConfigManager::new().unwrap();

        let mut output = Vec::new();

        let set_cli = Cli {
            command: Some(Commands::Config {
                command: ConfigCommand::Set {
                    key: ConfigKey::WatchDir,
                    value: Some("/new/watch/dir".to_string()),
                },
            }),
        };

        let set_result = {
            let mut writer = std::io::BufWriter::new(&mut output);
            set_cli.command.unwrap().execute(&mut writer)
        };
        assert!(set_result.is_ok(), "Config set command should execute successfully");

        let show_cli = Cli {
            command: Some(Commands::Config {
                command: ConfigCommand::Show,
            }),
        };

        let show_result = {
            let mut writer = std::io::BufWriter::new(&mut output);
            show_cli.command.unwrap().execute(&mut writer)
        };
        assert!(show_result.is_ok(), "Config show command should execute successfully");

        let show_output = String::from_utf8(output.clone()).unwrap();
        assert!(show_output.contains("Watch directory:"), "Output should contain Watch directory key");
        assert!(show_output.contains("/new/watch/dir"), "Output should contain the new watch directory");

        let clear_cli = Cli {
            command: Some(Commands::Config {
                command: ConfigCommand::Clear {
                    key: ConfigKey::WatchDir,
                },
            }),
        };

        let clear_result = {
            let mut writer = std::io::BufWriter::new(&mut output);
            clear_cli.command.unwrap().execute(&mut writer)
        };
        assert!(clear_result.is_ok(), "Config clear command should execute successfully");

        // Assert the config key was cleared
        let config = config_manager.load().unwrap();
        assert!(config.watch_dir.is_none(), "Watch directory should be cleared");
    }
}
