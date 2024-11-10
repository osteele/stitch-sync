use super::*;
use crate::types::machine::Machine;
use lazy_static::lazy_static;
use mockall::mock;
use mockall::predicate::*;
use std::path::PathBuf;
use std::sync::Mutex;

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
        fn get_all(&self) -> Result<Vec<Machine>, String>;
        fn get_by_name(&self, name: &str) -> Result<Option<Machine>, String>;
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
    fn test_watch_command() {
        let mut fs_mock = FILE_SYSTEM_MOCK.lock().unwrap();
        fs_mock.expect_create_dir().returning(|_| Ok(()));
        fs_mock.expect_write_file().returning(|_, _| Ok(()));
        fs_mock.expect_read_file().returning(|_| Ok("test".to_string()));
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
            .expect_get_by_name()
            .with(eq("test_machine"))
            .returning(|_| {
                Ok(Some(Machine::new(
                    "test_machine".to_string(),
                    vec![],
                    vec!["dst".to_string(), "exp".to_string()],
                    Some("/media/test".to_string()),
                    None,
                    None,
                )))
            });

        let cli = Cli {
            command: Some(Commands::Watch {
                dir: Some(PathBuf::from("/test/dir")),
                output_format: Some("exp".to_string()),
                machine: Some("test_machine".to_string()),
            }),
        };

        let result = cli.command.unwrap().execute();
        assert!(result.is_ok(), "Watch command should execute successfully");

        // Verify that the expected file system operations occurred
        fs_mock.checkpoint();  // Verifies all expected calls were made
        config_mock.checkpoint();
        machine_data_mock.checkpoint();
    }

    #[test]
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

        let result = cli.command.unwrap().execute();
        assert!(result.is_ok(), "Set machine command should execute successfully");

        // Verify that the config was updated as expected
        config_mock.checkpoint();
    }
}
