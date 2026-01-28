use std::env;

pub trait TtyChecker {
    fn stdout_is_tty(&self) -> bool;
    fn stdin_is_tty(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Plain,
    Pretty,
    InteractiveEligible,
}

pub struct CrosstermTty;

impl TtyChecker for CrosstermTty {
    fn stdout_is_tty(&self) -> bool {
        crossterm::tty::IsTty::is_tty(&std::io::stdout())
    }

    fn stdin_is_tty(&self) -> bool {
        crossterm::tty::IsTty::is_tty(&std::io::stdin())
    }
}

pub fn select_mode<T: TtyChecker>(checker: &T) -> Mode {
    if env_flag_set("MUSKETEER_PLAIN") || env_flag_set("CI") || !checker.stdout_is_tty() {
        return Mode::Plain;
    }

    if checker.stdin_is_tty() {
        return Mode::InteractiveEligible;
    }

    Mode::Pretty
}

fn env_flag_set(key: &str) -> bool {
    match env::var(key) {
        Ok(value) => !value.trim().is_empty() && value.trim() != "0",
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubTty {
        stdout: bool,
        stdin: bool,
    }

    impl TtyChecker for StubTty {
        fn stdout_is_tty(&self) -> bool {
            self.stdout
        }

        fn stdin_is_tty(&self) -> bool {
            self.stdin
        }
    }

    fn with_env_var<T>(key: &str, value: Option<&str>, f: impl FnOnce() -> T) -> T {
        let prior = env::var(key).ok();
        match value {
            Some(val) => env::set_var(key, val),
            None => env::remove_var(key),
        }
        let result = f();
        match prior {
            Some(val) => env::set_var(key, val),
            None => env::remove_var(key),
        }
        result
    }

    #[test]
    fn select_mode_plain_when_env_forces_plain() {
        with_env_var("MUSKETEER_PLAIN", Some("1"), || {
            let mode = select_mode(&StubTty {
                stdout: true,
                stdin: true,
            });
            assert_eq!(mode, Mode::Plain);
        });
    }

    #[test]
    fn select_mode_plain_when_stdout_not_tty() {
        with_env_var("MUSKETEER_PLAIN", None, || {
            let mode = select_mode(&StubTty {
                stdout: false,
                stdin: true,
            });
            assert_eq!(mode, Mode::Plain);
        });
    }

    #[test]
    fn select_mode_pretty_when_stdout_tty_stdin_not_tty() {
        with_env_var("MUSKETEER_PLAIN", None, || {
            with_env_var("CI", None, || {
                let mode = select_mode(&StubTty {
                    stdout: true,
                    stdin: false,
                });
                assert_eq!(mode, Mode::Pretty);
            });
        });
    }

    #[test]
    fn select_mode_interactive_when_tty_eligible() {
        with_env_var("MUSKETEER_PLAIN", None, || {
            with_env_var("CI", None, || {
                let mode = select_mode(&StubTty {
                    stdout: true,
                    stdin: true,
                });
                assert_eq!(mode, Mode::InteractiveEligible);
            });
        });
    }
}
