# se-cli
[![CI](https://github.com/derandomize/se-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/derandomize/se-cli/actions/workflows/ci.yml)

Простой интерпретатор командной строки на Rust. Проект в рамках курса по разработке ПО.

> Статус: реализованы REPL и выполнение команд (`cat`, `echo`, `wc`, `pwd`, `exit`),
> а также запуск внешних программ и поддержка кавычек/окружения (присваивания и передача env во внешний процесс).  
> **Пайпы и подстановки `$NAME` пока не реализованы.**

## Документация

- Архитектура: `docs/ARCHITECTURE.md`

## Что планируется поддержать (по заданию)

- **Builtins**: `cat`, `echo`, `wc`, `pwd`, `exit`
- **Кавычки**: `'...'` (full quoting) и `"..."` (weak quoting)
- **Окружение**: присваивания `NAME=value` и подстановка `$NAME`
- **External commands**: запуск через `PATH`, если команда не builtin
- **Пайплайны**: оператор `|`

## Пример использования

На текущем этапе реализованы REPL и команды без пайпов и подстановок:

```text
echo "Hello, world!"
cat README.md
wc README.md
pwd
FOO=bar cmd /C echo %FOO%        (Windows)
FOO=bar sh -c 'echo $FOO'        (Linux/macOS)
exit
```

## Сборка и запуск

- **Требования**: Rust stable (см. `Cargo.toml`, edition 2024).
- **Сборка**: `cargo build`
- **Запуск**: `cargo run`

Интерпретатор читает строки из stdin до `exit` или EOF.

## Разработка

- **Проверка форматирования**: `cargo fmt --all -- --check`
- **Линтинг**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Тесты**: `cargo test --all`

## Как помочь проекту (contributing)

Проект учебный, но вклад приветствуется:

- **Идеи/баги**: заведите issue в репозитории.
- **Pull request**: кратко опишите изменения и как их проверить.
- **Перед PR прогоните локально**:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all`

## Лицензия

MIT, см. файл `LICENSE`.

## Авторы

- Головачев Сергей
- Деружинский Дмитрий
- Токарев Алексей
