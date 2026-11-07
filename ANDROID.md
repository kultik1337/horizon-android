# Horizon for Android

An unofficial Android port of [Horizon](https://github.com/horizon-rust/horizon), the Rust
rewrite of Minecraft: Java Edition 26.3. Same game code, a NativeActivity APK around it, and
touch controls. Early: expect rough edges.

Horizon ships none of Mojang's files. You need your own copy of Minecraft: Java Edition, and
the game files are prepared from it on a computer.

## Get the APK

- From GitHub Actions: open the latest `android` run of this repository and download the
  `horizon-android-apk` artifact.
- Or build it yourself (below).

Install it on the phone (allow installs from unknown sources). arm64 phones only, Android 8 or
later, Vulkan or OpenGL ES 3.

## Put the game files on the phone

The game cannot prepare its files on the phone: preparing runs Mojang's Java code.

1. On a computer, install the Horizon launcher from playhorizon.dev, sign in with the Microsoft
   account that owns Java Edition, and start Horizon once. The launcher prepares the files.
2. Find the prepared folder: the one that holds `horizon-resources.json`. In a source checkout it
   is `.local/horizon-client/26.3`. In a launcher install it is the `resources` folder next to the
   game.
3. Start Horizon on the phone once, so it makes its folder, then connect the phone over USB and
   copy the whole folder's contents into
   `Android/data/dev.horizon.android/files/resources/`.
4. Start Horizon again.

Saves, `options.txt`, resource packs and logs live in `Android/data/dev.horizon.android/files/`.
You can copy a world from a computer into its `saves/` folder.

## Touch controls

| Where | What |
| --- | --- |
| Left lower part of the screen | Floating stick: put a finger down and slide to walk |
| Anywhere else, drag | Look around |
| Jump, Hit, Use | Jump, attack or break, use or place |
| Sneak, Run | Toggles |
| Inv | Inventory |
| `<` `>` | Previous and next hotbar slot |
| Drop, Chat, View, `||` | Drop item, chat, camera view, pause |

In menus a finger is the mouse pointer and a tap is a click. The buttons are controller buttons,
so the controller settings rebind them.

To try the touch layout on a desktop, start the game with `HORIZON_TOUCH=1`.

## Build

Needs the Android SDK (`platforms;android-35`, `build-tools;35.0.0`), NDK 28, a JDK and:

```sh
rustup target add aarch64-linux-android
cargo install cargo-ndk
export ANDROID_HOME=/path/to/sdk ANDROID_NDK_HOME=$ANDROID_HOME/ndk/28.2.13676358
packaging/android/build-apk.sh            # target/android/horizon-android.apk
```

## What changed from upstream

- `crates/horizon-android`: the `android_main` entry point; sets the game folder and
  `HORIZON_RESOURCES`, logs to logcat.
- `crates/horizon-client-input/src/platform.rs`: the event loop gets the Android activity.
- `crates/horizon-client-input/src/touch.rs`, `crates/horizon-client/src/game/touch.rs`: touch
  controls as a virtual controller, drawn over the HUD.
- `HORIZON_RESOURCES` as the default resources folder (`game.rs`, `Options::parse`).
- `packaging/android/`: manifest and APK script. `.github/workflows/android.yml` builds it; the
  upstream workflows are removed.

## Not done yet

- The soft keyboard for chat and sign text.
- Microsoft sign-in on the phone (online-mode servers). Single player, LAN and offline servers work
  without it.
- Discord presence, Play with Friends, the heartbeat and real controllers are left out of the Android build.
- Battery and heat: lower the render distance in Video Settings.
