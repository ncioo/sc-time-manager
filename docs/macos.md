# macOS

Код кроссплатформенный, отдельная ветка не нужна. Собирать нужно на самом Mac: кросс-компиляция с Windows непрактична из-за macOS SDK.

## Требования

- macOS и Xcode Command Line Tools:

  ```sh
  xcode-select --install
  ```

- [rustup](https://rustup.rs):

  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

## Сборка

```sh
git clone https://github.com/ncioo/sc-time-manager.git
cd sc-time-manager
cargo build --release
```

Первый build скачивает готовый ONNX Runtime (crate `ort`) — нужен интернет. Дальше сборка идёт из кэша.

## libonnxruntime.dylib

На Windows ONNX Runtime встроен в exe статически, поэтому получается один файл. На macOS он линкуется динамически, и рядом с бинарником должна лежать `libonnxruntime.dylib`: сборка кладёт её в `target/release`.

Для переноса скопируйте оба файла:

```sh
mkdir -p dist
cp target/release/sc-time-manager target/release/libonnxruntime.dylib dist/
```

Если dyld не находит библиотеку, укажите путь явно:

```sh
export DYLD_LIBRARY_PATH="$PWD/dist"
```

Или пропишите rpath:

```sh
install_name_tool -add_rpath @executable_path dist/sc-time-manager
```

Проверить зависимости бинарника можно через `otool -L dist/sc-time-manager`.

## Запуск

Приложение консольное: ему нужен терминал. Двойной клик по бинарнику обычно открывает Терминал, но надёжнее обёртка `sc-time-manager.command`:

```sh
#!/bin/sh
cd "$(dirname "$0")"
exec ./sc-time-manager
```

Не забудьте `chmod +x sc-time-manager.command`.

Файл выгрузки задаётся в `config.json` рядом с бинарником:

```json
"output_dir": "~/Work/Time Manager"
```

`~` — домашний каталог, папка создастся сама. Подробнее о настройках — в [config.md](config.md).

Буфер обмена читается через `pbpaste`; готовый `.xlsx` открывается через `open`, если `after_export` = `open`.

## Dock

Чтобы запускать трекер из Dock, соберите `.app` в Script Editor:

1. Откройте Script Editor (Программы → Утилиты) и создайте новый документ.
2. Вставьте:

   ```applescript
   tell application "Terminal"
       do script "cd '/Users/you/sc-time-manager/dist' && ./sc-time-manager"
       activate
   end tell
   ```

3. File → Export, формат **Application**, имя `Time Manager.app`.
4. Перетащите `.app` в Dock.

Путь и имя пользователя замените на свои. При первом запуске macOS может попросить разрешение на управление Terminal — разрешите в «Системные настройки → Конфиденциальность и безопасность → Автоматизация».

Как альтернатива, `.command`-файл можно перетащить в правую часть Dock — тогда он будет открывать Терминал напрямую, без `.app`.

### Размер окна

Размер задаётся профилем Terminal:

Terminal → Настройки → Профили → профиль по умолчанию → вкладка **Window** → **Columns: 120**, **Rows: 30**.

Новые окна будут открываться такого размера. Приложению нужно минимум 80×24.

### Иконка

1. Правый клик по `Time Manager.app` → **Get Info** (`Cmd+I`).
2. Перетащите `.icns` на маленькую иконку в левом верхнем углу — или скопируйте PNG и вставьте через `Cmd+V`.
3. Если Dock показывает старую иконку, уберите приложение из Dock и добавьте заново; помогает `killall Dock`.

Если нужен `.icns`, его можно собрать из PNG 1024×1024:

```sh
mkdir TimeManager.iconset
for s in 16 32 128 256 512; do
  sips -z $s $s icon.png --out "TimeManager.iconset/icon_${s}x${s}.png"
  sips -z $((s*2)) $((s*2)) icon.png --out "TimeManager.iconset/icon_${s}x${s}@2x.png"
done
iconutil -c icns TimeManager.iconset -o TimeManager.icns
```

При повторном экспорте `.app` из Script Editor иконку придётся назначить заново.

## Цвета

Terminal.app поддерживает 256 цветов, но не 24-битный RGB. Поэтому на macOS приложение автоматически переходит на палитру 256, если терминал не заявляет truecolor. Для полноценных цветов используйте iTerm2 или другой терминал с поддержкой 24 бит.

Режим можно задать вручную переменной `TIME_MANAGER_TRUECOLOR`: `1` — 24 бита, `0` — 256 цветов.

## Gatekeeper

Сборка не подписана и не нотаризована, поэтому Finder скажет «не удаётся проверить разработчика». Варианты:

- правый клик по файлу → «Открыть» → «Открыть»;
- снять карантин: `xattr -d com.apple.quarantine ./sc-time-manager`;
- для распространения — Apple Developer ID (99 $ в год) и нотаризация через `codesign` и `notarytool`.

## См. также

- [usage.md](usage.md) — вкладки, клавиши и работа с днём.
- [config.md](config.md) — `config.json`, выгрузка и темы.
