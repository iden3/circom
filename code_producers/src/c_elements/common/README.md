## Dependencies

You should have installed gcc and cmake

In ubuntu:

```sh
sudo apt install build-essential
sudo apt install cmake
```

## Compilation

### Compile witnesscalc for x86_64 host machine

```sh
make host
```

### Compile witnesscalc for arm64 host machine

```sh
make arm64_host
```

### Compile witnesscalc for Android

Install Android NDK from https://developer.android.com/ndk or with help of "SDK Manager" in Android Studio.

Set the value of ANDROID_NDK environment variable to the absolute path of Android NDK root directory.

Examples:

```sh
export ANDROID_NDK=/home/test/Android/Sdk/ndk/23.1.7779620  # NDK is installed by "SDK Manager" in Android Studio.
export ANDROID_NDK=/home/test/android-ndk-r23b              # NDK is installed as a stand-alone package.
```

Compilation for arm64:

```sh
make android
```

Compilation for x86_64:

```sh
make android_x86_64
```

### Compile witnesscalc for iOS

Requirements: Xcode.

1. Run:
    ````sh
    make ios
    ````
2. Open generated Xcode project.
3. Add compilation flag `-D_LONG_LONG_LIMB` to all build targets.
4. Add compilation flag `-DCIRCUIT_NAME=<your circuit name>` to the respective targets.
5. Compile witnesscalc.

## License

witnesscalc is part of the iden3 project copyright 2022 0KIMS association and published with GPL-3 license. Please check the COPYING file for more details.
