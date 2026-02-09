//! Реализация встроенных команд.

use super::types::{IoStreams, ShellControl, ShellError, ShellResult};

/// Перечисление встроенных команд, поддерживаемых на этом этапе.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Builtin {
    Cat,
    Echo,
    Wc,
    Pwd,
    Exit,
}

impl Builtin {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "cat" => Some(Builtin::Cat),
            "echo" => Some(Builtin::Echo),
            "wc" => Some(Builtin::Wc),
            "pwd" => Some(Builtin::Pwd),
            "exit" => Some(Builtin::Exit),
            _ => None,
        }
    }
}

/// Выполняет builtin-команду.
pub(crate) fn run_builtin(
    builtin: Builtin,
    args: &[String],
    io: &mut IoStreams<'_>,
) -> ShellResult<ShellControl> {
    match builtin {
        Builtin::Echo => run_echo(args, io),
        Builtin::Pwd => run_pwd(io),
        Builtin::Exit => run_exit(args),
        Builtin::Cat => run_cat(args, io),
        Builtin::Wc => run_wc(args, io),
    }
}

fn run_echo(args: &[String], io: &mut IoStreams<'_>) -> ShellResult<ShellControl> {
    if !args.is_empty() {
        write!(io.stdout, "{}", args.join(" ")).map_err(ShellError::Io)?;
    }
    writeln!(io.stdout).map_err(ShellError::Io)?;
    Ok(ShellControl::Continue(0))
}

fn run_pwd(io: &mut IoStreams<'_>) -> ShellResult<ShellControl> {
    let dir = std::env::current_dir().map_err(ShellError::Io)?;
    writeln!(io.stdout, "{}", dir.display()).map_err(ShellError::Io)?;
    Ok(ShellControl::Continue(0))
}

fn run_exit(args: &[String]) -> ShellResult<ShellControl> {
    let code = args
        .first()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    Ok(ShellControl::Exit(code))
}

fn run_cat(args: &[String], io: &mut IoStreams<'_>) -> ShellResult<ShellControl> {
    if args.is_empty() {
        writeln!(io.stderr, "cat: missing file operand").map_err(ShellError::Io)?;
        return Ok(ShellControl::Continue(2));
    }

    let mut exit_code = 0;
    for path in args {
        match std::fs::read(path) {
            Ok(bytes) => {
                io.stdout.write_all(&bytes).map_err(ShellError::Io)?;
            }
            Err(e) => {
                writeln!(io.stderr, "cat: {path}: {e}").map_err(ShellError::Io)?;
                exit_code = 1;
            }
        }
    }
    Ok(ShellControl::Continue(exit_code))
}

fn run_wc(args: &[String], io: &mut IoStreams<'_>) -> ShellResult<ShellControl> {
    if args.len() != 1 {
        writeln!(io.stderr, "wc: expected exactly one file path").map_err(ShellError::Io)?;
        return Ok(ShellControl::Continue(2));
    }
    let path = &args[0];

    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            writeln!(io.stderr, "wc: {path}: {e}").map_err(ShellError::Io)?;
            return Ok(ShellControl::Continue(1));
        }
    };

    let byte_count = bytes.len();
    let text = String::from_utf8_lossy(&bytes);
    let line_count = text.lines().count();
    let word_count = text.split_whitespace().count();

    writeln!(io.stdout, "{line_count} {word_count} {byte_count}").map_err(ShellError::Io)?;
    Ok(ShellControl::Continue(0))
}
