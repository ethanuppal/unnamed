# unnamed:  Extremely simple window layout engine for macOS

> [!NOTE]
> The name, in addition to the code, is a work-in-progress!

This is currently just for me because I want to automatically pin apps to have specific border insets.
Thus, the border insets are not configurable -- they will always be

```rs
const LEFT_INSET: CGFloat = 8.0;
const RIGHT_INSET: CGFloat = 8.0;
const TOP_INSET: CGFloat = 6.0;
const BOTTOM_INSET: CGFloat = 8.0;
const INNER_SPACING: CGFloat = 12.0;
```

because those look best on my system.
**It's also probably very buggy.**

## Usage

First, clone:

```shell
git clone https://github.com/ethanuppal/unnamed.git
cd unnamed
```

Then, run:

```shell
cargo run --release -- <bundle IDs>
```

For example:

```shell
cargo run --release -- com.apple.Safari net.kovidgoyal.kitty
```

You will need to give `target/release/unnamed` accessibility permissions.

> [!CAUTION]
> Right now, I don't check for whether windows get resized afterward --- the next step is to (1) setup `AXObserver`s for when new windows are created, get moved, or get resized and (2) setup `NSNotification`s for when the specified apps are closed and reopened.

## Layouts

Three layout options are supported:

- full screen
- left
- right

<!--## Move windows around-->
<!---->
<!--You can use RPC to port 12345:-->
<!---->
<!--```shell-->
<!--curl -X POST http://localhost:12345 \-->
<!--    -H "Content-Type: application/json" \-->
<!--    -d '{"bundleID": "net.kovidgoyal.kitty", "position": "left"}'-->
<!--```-->
<!---->
<!--Pass the bundle ID and the position (one of `"left"`, `"full"`, or `"right"`).-->
