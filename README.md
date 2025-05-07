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
cargo build --release
./target/release/unnamed -- apps.txt
```

For example, see [`apps.txt`](./apps.txt).
This file contains the "core" apps --- these are the apps that unnamed will actively maintain in certain layouts (_i,e,._, you can't accidently move or resize them).

You will need to give `target/release/unnamed` accessibility permissions.

> [!CAUTION]
> Right now, I don't check for whether windows get resized afterward --- the next step is to (1) setup `AXObserver`s for when new windows are created, ~~get moved, or get resized~~ and (2) setup `NSNotification`s for when the specified apps are closed and reopened.

## Roadmap

- [x] Layout apps on screen
- [x] Keybinds to change app layout or prevent maintaining layout
- [x] Non-core apps can still be manually/temporarily layouted with keybinds
- [ ] Check when a core app launches new windows and handle those / check when a core app closes a window
- [ ] Check when a core app is closed and reopened

## Layouts

Three layout options are supported (where `Super` is `Command-Control-Option`):

- full screen (`Super-Shift-C`)
- left (`Super-Shift-H`)
- right (`Super-Shift-L`)
- toggle floating (`Super-Shift-Space`)

The toggle-floating option only matters for core apps since non-core apps will always behave as if they are in that state.

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

## Debugging

This is for me on macOS:

Address sanitizer:

```shell
ASAN_OPTIONS=detect_leaks=1:symbolize=1 RUSTFLAGS=-Zsanitizer=address cargo +nightly r -Z build-std --target aarch64-apple-darwin -- com.apple.Safari net.kovidgoyal.kitty
```
