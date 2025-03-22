# wise: Automatic window resizer for macOS

> [!NOTE]
> The previous (working) version of this tool is on the `main` branch.
> Development is now taking place on the `rust` branch.

This is currently just for me because I want to automatically pin apps to have specific border insets.
Thus, the border insets are not configurable -- they will always be 8, 8, 6, 8 beecause those look best on my system.
**It's also probably very buggy.**

## Usage

First, clone:

```shell
git clone https://github.com/ethanuppal/wise.git
cd wise
```

Then, run:

```shell
cargo run --release -- <bundle ids>
```

For example:

```shell
cargo run --release -- com.apple.Safari net.kovidgoyal.kitty
```

## Move windows around

You can use RPC to port 12345:

```shell
curl -X POST http://localhost:12345 \
    -H "Content-Type: application/json" \
    -d '{"bundleID": "net.kovidgoyal.kitty", "position": "left"}'
```

Pass the bundle ID and the position (one of `"left"`, `"full"`, or `"right"`).
