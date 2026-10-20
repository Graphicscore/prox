#!/usr/bin/env bash
# Source this before any `npm run tauri android ...` command: `source scripts/android-env.sh` (bash or zsh)
export JAVA_HOME=/usr/lib/jvm/java-21-openjdk   # Gradle/AGP do not support the default JDK 27
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
NDK_VERSION="$(ls "$ANDROID_HOME/ndk" 2>/dev/null | sort -V | tail -1)"
export NDK_HOME="$ANDROID_HOME/ndk/$NDK_VERSION"   # dangling until the NDK is installed
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$HOME/.cargo/bin:$PATH"
