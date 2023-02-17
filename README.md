# Sura

Text editor. Sura Sura.

## Usage

### Running

```
cargo run "file path"
```

### Configuration

Sura uses [XDG Base Directory](https://specifications.freedesktop.org/basedir-spec/latest/) to store a configuration file.

In `$XDG_CONFIG_HOME/sura/config.json`,
```json
{
    "languageServers": {
        "language": "language server path"
    }
}
```

### Keyboard commands

| Keys       | Function                 |
| ---------- | ------------------------ |
| Arrow keys | Move cursor              |
| Ctrl + N   | Go forward to next page  |
| Ctrl + B   | Go back to previous page |
| Ctrl + S   | Save                     |
| Ctrl + Q   | Quit                     |
