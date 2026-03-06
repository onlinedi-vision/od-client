# Building OnlineDivision for Android
## Prerequisites
* Android Studio (obviously)
* Latest Android SDK and NDK (managed via the SDK manager built into Android Studio)
* Rust android build targets: `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`
## Guide
One-time setup: Run `npx tauri android init` to prepare your local dev environment for android building. This should only need to be used once.

Set environment variables:
`JAVA_HOME`: Link to your Java installation root.
`ANDROID_NDK_ROOT`: The Android NDK install location, on Windows:`%localappdata%\Android\Sdk\ndk\<version>`
On Linux: normally `~/Android/Sdk/ndk/<version>/`

Build initial not-yet-executable APK: `npx tauri android build`
The APK will be dropped in `src-tauri/gen/android/app/build/outputs/apk/universal/release`.

Align the archive: 
`zipalign -p -v 4 app-universal-release-unsigned.apk app-aligned.apk`
If it's not in PATH after installing Android Studio, zipalign should be found in side the SDK folder, under build-tools.

Create a key store (only needs to be done once):
`keytool -genkey -v -keystore <key_name>.jks -keyalg RSA -keysize 2048 -validity 10000 -alias <key_alias_dont_forget>`
Follow the prompts after you start the program and it should create the keys. We don't have an 'official' key yet since we're just testing so just enter whatever. Keytool should be located in your Java home. Don't keep the key store in `src-tauri/gen`, keep it somewhere else on your PC.

Sign the APK archive:
` apksigner sign --ks <key_name>.jks --ks-key-alias <key_alias_dont_forget> app-aligned.apk`
You will be prompted to enter the same password you used when creating the keys. Since we're just testing, if you forget it just re-generate a new key store.

After all this, if everything went right, the aligned apk should be executable. You can test it by trying to run it on an Android Studio virtual machine.
