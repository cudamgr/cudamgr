#[cfg(test)]
mod tests {
    use crate::cli::commands::*;

    #[test]
    fn test_install_args_validation() {
        let args = InstallArgs {
            version: "11.8".to_string(),
            force: false,
            skip_driver: false,
        };
        assert!(args.validate().is_ok());

        let args = InstallArgs {
            version: "".to_string(),
            force: false,
            skip_driver: false,
        };
        assert!(args.validate().is_err());

        let args = InstallArgs {
            version: "invalid-version!".to_string(),
            force: false,
            skip_driver: false,
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_use_args_validation() {
        let args = UseArgs {
            version: "12.0".to_string(),
            install: false,
        };
        assert!(args.validate().is_ok());

        let args = UseArgs {
            version: "".to_string(),
            install: false,
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_logs_args_validation() {
        let args = LogsArgs {
            lines: 50,
            follow: false,
        };
        assert!(args.validate().is_ok());

        let args = LogsArgs {
            lines: 0,
            follow: false,
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_doctor_args_validation() {
        let args = DoctorArgs { verbose: true };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_command_handlers_exist() {
        // Test that all handlers can be created
        let _doctor_handler = DoctorHandler::new(DoctorArgs { verbose: false });
        let _install_handler = InstallHandler::new(InstallArgs {
            version: "11.8".to_string(),
            force: false,
            skip_driver: false,
        });
        let _use_handler = UseHandler::new(UseArgs {
            version: "11.8".to_string(),
            install: false,
        });
        let _list_handler = ListHandler::new(ListArgs {
            available: false,
            verbose: false,
        });
        let _uninstall_handler = UninstallHandler::new(UninstallArgs {
            version: "11.8".to_string(),
            yes: false,
        });
        let _logs_handler = LogsHandler::new(LogsArgs {
            lines: 50,
            follow: false,
        });

        // Just verify they can be created (compilation test)
        assert!(true);
    }
}

#[cfg(test)]
mod output_tests {
    use crate::cli::output::*;

    #[test]
    fn test_progress_bar_creation() {
        let _progress = ProgressBar::new(100, "Test operation".to_string());
        // Test that it can be created without panicking
        assert!(true);
    }

    #[test]
    fn test_progress_bar_update() {
        let mut progress = ProgressBar::new(100, "Test operation".to_string());
        progress.update(50);
        // Test that update doesn't panic
        assert!(true);
    }

    #[test]
    fn test_spinner_creation() {
        let _spinner = Spinner::new("Loading...".to_string());
        // Test that it can be created without panicking
        assert!(true);
    }
}