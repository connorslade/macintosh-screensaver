# Macintosh Screensaver

https://github.com/user-attachments/assets/b7b680d0-50c3-4ec0-8888-467fd0a8990b

My attempt at a cross-platform recreation of the MacOS Sequoia ["Macintosh" screensaver](https://basicappleguy.com/haberdashery/macintoshwallpapers).

The screensaver is available as a regular windowed application through the binary of the main crate, or as a wayland client supporting the layer-shell extension.
The layer shell version can be used as a lock screen on sway with [swaylock-plugin](https://github.com/mstoeckl/swaylock-plugin).

## Todo

- [x] animated images
- [x] make a bunch of scenes
- [x] fix color palette
- [ ] make pixels shader image size / scale independent
- [x] login screen?
- [x] embed default resources
- [ ] expose cli
- [ ] randomize starting points (colormap & scene) (also randomize scene order?)
- [ ] drop shadow
