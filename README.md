> [!WARNING]  
> This app is still in it's infancy and it is NOT production ready!

## division.client

The official client for the division online app - a cross-platform GPL-free and open-source VoIP and Instant Messaging desktop app.

### Installing 

Installing the current version of the division.client app can be done in just a few steps!

#### Installing needed programs
- [npm](https://www.npmjs.com/)
- [cargo](https://www.rust-lang.org/tools/install)
- [git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git) (if not already installed)
#### Building the app from source
To be able to build the app from source, you will first need to clone this repo:
```sh
$ git clone 'https://github.com/onlinedi-vision/od-client.git'
$ cd od-client
```
After that, building the app is just as simple as running:
```sh 
$ npm run tauri build
$ # or for a debug version:
$ npm run tauri dev
```
(Keep in mind that this might take a couple of minutes, and getting an error related to "linuxdeploy" is completely normal and can be ignored)
### Usage
The previous command should state the location to the "division.client" executable. But - just in case - the file is expected to be found at "./src-tauri/target/release/division-client". As such, to run the app you should run the following command:
```sh 
$ ./src-tauri/target/realease/division-client
```
To make it more accesible you could also copy it to one of your /bin directories. For example:
```sh 
$ cp ./src-tauri/target/release/divisiona /usr/bin/division-client
```
Now you should be able to run it more easily:
```sh 
$ division-client 
```
### Using nix-shell
Run the following
```sh
$ nix-shell shell.nix
[nix-shell]$ npm run tauri dev 
```
On wayland you may need to set the following env vars:
```sh
[nix-shell]$ WEBKIT_DISABLE_DMABUF_RENDERER=1 WAYLAND_DISPLAY= WEBKIT_DISABLE_COMPOSITING_MODR=1 npm run tauri dev
```

### Troubleshooting
1. "Failed to create GBM buffer of size 945x1044: Invalid argument".
If you have and nVidia graphics card and use prime offloading you should be able to fix this by setting the __NV_PRIME_RENDER_OFFLOAD environment variable to 1 every time you run the app:
```sh 
$ __NV_PRIME_RENDER_OFFLOAD=1 division-client 
```
(to make this easier you could also set a bash alias for the command ```alias do-client="__NV_PRIME_RENDER_OFFLOAD=1 division-client"```). <br>
This problem could also happen on the quick install version (i.e. after installing the client with the i.sh script). If you install the client using:
```sh 
$ wget -qO- https://onlinedi.vision/cdn/install/i.sh | bash
```
you might get the same problem. In this case you could edit the:
```sh 
/usr/bin/division-online-client 
```
file to add ```__NV_PRIME_RENDER_OFFLOAD=1``` at the beginning of the second line. The script file should look like this after the edit:
```
#!/bin/bash 
__NV_PRIME_RENDER_OFFLOAD=1 /opt/division-online/division.client.AppDir/AppRun
```

2. "failed to run linuxdeploy" during Tauri Build. 
This is a problem that has not been fully fixed by the Tauri devs. You can try running the build command using the ```NO_STRIP=true``` environment variable as follows:
```sh 
$ NO_STRIP=true npm run tauri build
```
This might still not fix the issue, so if it still doesn't work keep in mind that the appimage could've still been built. So you should still be able to find your appimage at the path I've described before.

3. Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display
This may happen on wayland. To fix this try running the client using these env vars:
```sh
$ WEBKIT_DISABLE_DMABUF_RENDERER=1 WAYLAND_DISPLAY= WEBKIT_DISABLE_COMPOSITING_MODR=1 division-client
```
