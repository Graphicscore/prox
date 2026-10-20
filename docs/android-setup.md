# Android toolchain setup

One-time setup for building the Android app. Every command is a single line; paste them one at a time.

## Commands

| # | Command | Needs | Why |
|---|---------|-------|-----|
| 1 | `sudo pacman -S rustup` | sudo, interactive | rustup replaces Arch's `rust` package (accept the removal prompt); Arch's rust cannot add Android targets. |
| 2 | `rustup default stable` | - | Installs the stable toolchain under rustup. |
| 3 | `rustup target add aarch64-linux-android x86_64-linux-android i686-linux-android` | - | Rust std for the Android ABIs (arm64 devices, x86_64 and x86 emulators). |
| 4 | `JAVA_HOME=/usr/lib/jvm/java-21-openjdk ~/Android/Sdk/cmdline-tools/latest/bin/sdkmanager --licenses` | interactive (answer `y`) | The SDK refuses to install packages until licenses are accepted; JDK 21 because sdkmanager breaks on the default JDK 27. |
| 5 | `JAVA_HOME=/usr/lib/jvm/java-21-openjdk ~/Android/Sdk/cmdline-tools/latest/bin/sdkmanager "ndk;28.2.13676358"` | - | The NDK provides the clang linker Rust needs for Android targets (NDK 28.2 chosen conservatively; 29.x and 30.x also exist but are newer than most Tauri/Rust setups are tested with). |

## Use

Run `source scripts/android-env.sh` in every shell before `npm run tauri android ...` (works in bash and zsh). It sets `JAVA_HOME` (JDK 21), `ANDROID_HOME`, `NDK_HOME` (newest dir under `$ANDROID_HOME/ndk`) and `PATH`.

## Verify

```
bash -c 'source scripts/android-env.sh && java -version 2>&1 | head -1 && rustup target list --installed && test -d "$NDK_HOME" && echo NDK_OK'
```

Expected: `openjdk version "21...`, the three `*-linux-android` targets, then `NDK_OK`.

## Troubleshooting

| Symptom | Cause / fix |
|---------|-------------|
| `rustup: command not found` | Step 1 not done, or `~/.cargo/bin` not on PATH: run `source scripts/android-env.sh`. |
| `java -version` shows 27 (or Gradle/sdkmanager errors about the JDK) | Env script not sourced; JDK 21 is required. Check `/usr/lib/jvm/java-21-openjdk` exists (`sudo pacman -S jdk21-openjdk`). |
| `NDK_HOME` empty or `.../ndk/` | NDK not installed yet (step 5); re-source the script afterwards. |
