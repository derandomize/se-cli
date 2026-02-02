# se-cli
[![CI](https://github.com/derandomize/se-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/derandomize/se-cli/actions/workflows/ci.yml)

Простой интерпретатор командной строки на Rust. Проект в рамках курса по разработке ПО.

> Статус: на текущей стадии в репозитории находится **архитектурная документация (задание 1)**.  
> Реализация интерпретатора будет добавлена в следующих заданиях.

## Документация

- Архитектура: `docs/ARCHITECTURE.md`

## Что планируется поддержать (по заданию)

- **Builtins**: `cat`, `echo`, `wc`, `pwd`, `exit`
- **Кавычки**: `'...'` (full quoting) и `"..."` (weak quoting)
- **Окружение**: присваивания `NAME=value` и подстановка `$NAME`
- **External commands**: запуск через `PATH`, если команда не builtin
- **Пайплайны**: оператор `|`

## Пример использования

Пока **не реализовано** (архитектурная стадия).

## Сборка и запуск

Пока **не реализовано** (архитектурная стадия).

- **Требования**: Rust stable (см. `Cargo.toml`, edition 2024).
- **Сборка**: `cargo build`
- **Запуск**: `cargo run`

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
