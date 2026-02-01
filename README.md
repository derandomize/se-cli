# se-cli
Простой интерпретатор командной строки на Rust. Проект в рамках курса по разработке ПО.

## Документация

- Архитектура: `docs/ARCHITECTURE.md`

## Что планируется поддержать (по заданию)

- **Builtins**: `cat`, `echo`, `wc`, `pwd`, `exit`
- **Кавычки**: `'...'` (full quoting) и `"..."` (weak quoting)
- **Окружение**: присваивания `NAME=value` и подстановка `$NAME`
- **External commands**: запуск через `PATH`, если команда не builtin
- **Пайплайны**: оператор `|`

## Разработка

- **Проверка форматирования**: `cargo fmt --all -- --check`
- **Линтинг**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Тесты**: `cargo test --all`
