//! Реализация встроенных команд.

use clap::Parser;
use regex::Regex;
use regex::RegexBuilder;

use super::types::{IoStreams, ShellControl, ShellError, ShellResult};

fn io_error_message(e: &std::io::Error) -> String {
    // `std::io::Error` форматируется так: "No such file or directory (os error 2)".
    // Для консистентности сообщений убираем числовой суффикс ОС.
    let s = e.to_string();
    match s.split_once(" (os error") {
        Some((prefix, _)) => prefix.to_string(),
        None => s,
    }
}

/// Перечисление встроенных команд, поддерживаемых на этом этапе.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Builtin {
    Cat,
    Echo,
    Grep,
    Wc,
    Pwd,
    Exit,
}

impl Builtin {
    /// Возвращает builtin по имени команды (если она поддерживается).
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "cat" => Some(Builtin::Cat),
            "echo" => Some(Builtin::Echo),
            "grep" => Some(Builtin::Grep),
            "wc" => Some(Builtin::Wc),
            "pwd" => Some(Builtin::Pwd),
            "exit" => Some(Builtin::Exit),
            _ => None,
        }
    }
}

/// Выполняет builtin-команду.
///
/// Возвращает:
/// - `ShellControl::Continue(code)` для продолжения REPL (где `code` — exit code команды)
/// - `ShellControl::Exit(code)` для завершения REPL
pub(crate) fn run_builtin(
    builtin: Builtin,
    args: &[String],
    io: &mut IoStreams<'_>,
) -> ShellResult<ShellControl> {
    run_builtin_with_input(builtin, args, None, io)
}

/// Выполняет builtin-команду, опционально получая stdin (для пайпов).
///
/// На этом этапе stdin используется только для тех команд, для которых это нужно
/// в пайплайнах (например, `wc` без аргументов).
pub(crate) fn run_builtin_with_input(
    builtin: Builtin,
    args: &[String],
    stdin: Option<&[u8]>,
    io: &mut IoStreams<'_>,
) -> ShellResult<ShellControl> {
    match builtin {
        Builtin::Echo => run_echo(args, io),
        Builtin::Pwd => run_pwd(io),
        Builtin::Exit => run_exit(args),
        Builtin::Cat => run_cat(args, stdin, io),
        Builtin::Grep => run_grep(args, stdin, io),
        Builtin::Wc => run_wc(args, stdin, io),
    }
}

/// Печатает аргументы, разделяя их пробелами, и перевод строки в конце.
fn run_echo(args: &[String], io: &mut IoStreams<'_>) -> ShellResult<ShellControl> {
    if !args.is_empty() {
        write!(io.stdout, "{}", args.join(" ")).map_err(ShellError::Io)?;
    }
    writeln!(io.stdout).map_err(ShellError::Io)?;
    Ok(ShellControl::Continue(0))
}

/// Печатает текущую рабочую директорию и перевод строки.
fn run_pwd(io: &mut IoStreams<'_>) -> ShellResult<ShellControl> {
    let dir = std::env::current_dir().map_err(ShellError::Io)?;
    writeln!(io.stdout, "{}", dir.display()).map_err(ShellError::Io)?;
    Ok(ShellControl::Continue(0))
}

/// Завершает REPL.
///
/// Если указан аргумент, он трактуется как код возврата (i32). Некорректный аргумент -> 0.
fn run_exit(args: &[String]) -> ShellResult<ShellControl> {
    let code = args
        .first()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    Ok(ShellControl::Exit(code))
}

/// Выводит содержимое файлов подряд.
///
/// Коды возврата:
/// - 0: все файлы прочитаны успешно
/// - 1: хотя бы один файл не прочитан
/// - 2: не передан ни один путь
fn run_cat(
    args: &[String],
    stdin: Option<&[u8]>,
    io: &mut IoStreams<'_>,
) -> ShellResult<ShellControl> {
    if args.is_empty() {
        if let Some(input) = stdin {
            io.stdout.write_all(input).map_err(ShellError::Io)?;
            return Ok(ShellControl::Continue(0));
        } else {
            writeln!(io.stderr, "cat: missing file operand").map_err(ShellError::Io)?;
            return Ok(ShellControl::Continue(2));
        }
    }

    let mut exit_code = 0;
    for path in args {
        match std::fs::read(path) {
            Ok(bytes) => {
                io.stdout.write_all(&bytes).map_err(ShellError::Io)?;
            }
            Err(e) => {
                let msg = io_error_message(&e);
                writeln!(io.stderr, "cat: {path}: {msg}").map_err(ShellError::Io)?;
                exit_code = 1;
            }
        }
    }
    Ok(ShellControl::Continue(exit_code))
}

/// Печатает количество строк/слов/байт для одного файла.
///
/// Формат вывода: `<lines> <words> <bytes>`.
///
/// Коды возврата:
/// - 0: успех
/// - 1: ошибка чтения файла
/// - 2: неверное число аргументов
fn run_wc(
    args: &[String],
    stdin: Option<&[u8]>,
    io: &mut IoStreams<'_>,
) -> ShellResult<ShellControl> {
    if args.is_empty() {
        if let Some(input) = stdin {
            let (line_count, word_count, byte_count) = count_wc(input);
            writeln!(io.stdout, "{line_count} {word_count} {byte_count}")
                .map_err(ShellError::Io)?;
            return Ok(ShellControl::Continue(0));
        }
        writeln!(io.stderr, "wc: missing file operand").map_err(ShellError::Io)?;
        return Ok(ShellControl::Continue(2));
    }
    if args.len() != 1 {
        writeln!(io.stderr, "wc: expected exactly one file path").map_err(ShellError::Io)?;
        return Ok(ShellControl::Continue(2));
    }
    let path = &args[0];

    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            let msg = io_error_message(&e);
            writeln!(io.stderr, "wc: {path}: {msg}").map_err(ShellError::Io)?;
            return Ok(ShellControl::Continue(1));
        }
    };

    let (line_count, word_count, byte_count) = count_wc(&bytes);

    writeln!(io.stdout, "{line_count} {word_count} {byte_count}").map_err(ShellError::Io)?;
    Ok(ShellControl::Continue(0))
}

fn count_wc(bytes: &[u8]) -> (usize, usize, usize) {
    let byte_count = bytes.len();
    let text = String::from_utf8_lossy(bytes);
    let line_count = text.lines().count();
    let word_count = text.split_whitespace().count();
    (line_count, word_count, byte_count)
}

#[derive(Parser, Debug)]
#[command(name = "grep", disable_help_flag = true, disable_version_flag = true)]
struct GrepCli {
    /// Search only for whole words.
    #[arg(short = 'w')]
    word: bool,

    /// Case-insensitive search.
    #[arg(short = 'i')]
    ignore_case: bool,

    /// Print NUM lines of trailing context after matching lines.
    #[arg(short = 'A', value_name = "NUM", default_value_t = 0)]
    after: usize,

    /// Regular expression pattern.
    pattern: String,

    /// Files to search. If omitted, grep reads from stdin (pipeline input).
    files: Vec<String>,
}

/// Печатает строки, которые матчатся по regex-шаблону.
///
/// Поддерживаемые флаги:
/// - `-w`: совпадение только по целому слову (границы слова определяем как не-`[\p{L}\p{N}_]`)
/// - `-i`: регистронезависимый поиск
/// - `-A N`: печатать N строк после совпадения (пересекающиеся области не дублируются)
///
/// Коды возврата (как в grep):
/// - 0: найдено хотя бы одно совпадение
/// - 1: совпадений нет
/// - 2: ошибка аргументов/regex/чтения
fn run_grep(
    args: &[String],
    stdin: Option<&[u8]>,
    io: &mut IoStreams<'_>,
) -> ShellResult<ShellControl> {
    let argv = std::iter::once("grep".to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>();

    let parsed = match GrepCli::try_parse_from(argv) {
        Ok(p) => p,
        Err(e) => {
            writeln!(io.stderr, "grep: {e}").map_err(ShellError::Io)?;
            return Ok(ShellControl::Continue(2));
        }
    };

    let re = match build_regex(&parsed.pattern, parsed.ignore_case) {
        Ok(r) => r,
        Err(msg) => {
            writeln!(io.stderr, "grep: {msg}").map_err(ShellError::Io)?;
            return Ok(ShellControl::Continue(2));
        }
    };

    let mut found_any = false;
    let mut had_error = false;

    // Если файлы не заданы — читаем из stdin.
    if parsed.files.is_empty() {
        let Some(input) = stdin else {
            writeln!(io.stderr, "grep: missing file operand").map_err(ShellError::Io)?;
            return Ok(ShellControl::Continue(2));
        };

        let found = grep_bytes_into_output(&re, parsed.word, parsed.after, None, input, io)?;
        found_any |= found;
    } else {
        let prefix = parsed.files.len() > 1;
        for path in &parsed.files {
            match std::fs::read(path) {
                Ok(bytes) => {
                    let found = grep_bytes_into_output(
                        &re,
                        parsed.word,
                        parsed.after,
                        if prefix { Some(path.as_str()) } else { None },
                        &bytes,
                        io,
                    )?;
                    found_any |= found;
                }
                Err(e) => {
                    let msg = io_error_message(&e);
                    writeln!(io.stderr, "grep: {path}: {msg}").map_err(ShellError::Io)?;
                    had_error = true;
                }
            }
        }
    }

    let code = if had_error {
        2
    } else if found_any {
        0
    } else {
        1
    };
    Ok(ShellControl::Continue(code))
}

fn grep_bytes_into_output(
    re: &Regex,
    word: bool,
    after: usize,
    file_prefix: Option<&str>,
    bytes: &[u8],
    io: &mut IoStreams<'_>,
) -> ShellResult<bool> {
    let text = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = text.lines().collect();

    let mut found = false;
    let mut print_until: isize = -1;
    for (idx, line) in lines.iter().enumerate() {
        let is_match = if word {
            line_has_whole_word_match(re, line)
        } else {
            re.is_match(line)
        };

        if is_match {
            found = true;
            let end = idx.saturating_add(after) as isize;
            if end > print_until {
                print_until = end;
            }
        }

        if (idx as isize) <= print_until {
            if let Some(prefix) = file_prefix {
                writeln!(io.stdout, "{prefix}:{line}").map_err(ShellError::Io)?;
            } else {
                writeln!(io.stdout, "{line}").map_err(ShellError::Io)?;
            }
        }
    }

    Ok(found)
}

fn build_regex(pattern: &str, ignore_case: bool) -> Result<Regex, String> {
    RegexBuilder::new(pattern)
        .case_insensitive(ignore_case)
        .build()
        .map_err(|e| format!("invalid regex: {e}"))
}

fn is_word_constituent(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn line_has_whole_word_match(re: &Regex, line: &str) -> bool {
    for m in re.find_iter(line) {
        if m.start() == m.end() {
            continue;
        }

        let before = line[..m.start()].chars().last();
        let after = line[m.end()..].chars().next();

        let ok_before = before.is_none_or(|c| !is_word_constituent(c));
        let ok_after = after.is_none_or(|c| !is_word_constituent(c));

        if ok_before && ok_after {
            return true;
        }
    }
    false
}
