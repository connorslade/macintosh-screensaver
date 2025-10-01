# Macintosh Screensaver

https://github.com/user-attachments/assets/b7b680d0-50c3-4ec0-8888-467fd0a8990b

My attempt at a cross-platform recreation of the MacOS Sequoia ["Macintosh" screensaver](https://basicappleguy.com/haberdashery/macintoshwallpapers).

The screensaver is available as a regular windowed application through the binary of the main crate, or as a wayland client supporting the layer-shell extension.
The layer shell version can be used as a lock screen on sway with [swaylock-plugin](https://github.com/mstoeckl/swaylock-plugin).

## Configuration

By default the default configuration bundled into the executables will be used.
The config defines the colormap, and the images / keyframes making up each scene.

Right now you have to edit the source code to repackage config (i dont really feel like making a CLI rn).
But anyway once you make whatever edits to anything in the [animation](animation) directory, add the following to the top of the [main crate's `main` function](https://github.com/connorslade/macintosh-screensaver/blob/7f393c1d32c426c412fc7fd2233ed31ac4569a81/src/main.rs#L34).

```rust
let animation = Animation::load_dev("animation/animation.toml").unwrap();
animation.export("animation/animation.bin").unwrap();
```

After running (`cargo r -r`) this will replace `animation.bin` with a new version built from your config.
Now remove the new code and after recompiling again, the new animation config will be packaged.

## Usage

After cloning the repository, just build with cargo.

```bash
git clone https://github.com/connorslade/macintosh-screensaver
cd macintosh-screensaver
cargo b -r
```

If you want to use as a lock screen on sway, first install `swaylock-plugin`, then run the following commands.

```bash
# might be a good idea to rename the output to macintosh-screensaver-layer or smth
cargo b -r -p wayland
swaylock-plugin --command path/to/macintosh-screensaver-layer
```

## Todo

- [x] animated images
- [x] make a bunch of scenes
- [x] fix color palette
- [x] make pixels shader image size / scale independent
- [x] login screen?
- [x] embed default resources
- [ ] expose cli
- [x] randomize starting points (colormap & scene)
- [ ] randomize scene order?
- [ ] drop shadow

<details>
<summary><a href="https://infinitemac.org">Infinite Mac</a> Screen Recorder</summary>

```js
function imageDiff(a, b) {
  if (a.width != b.width || a.height != b.height) return true;
  for (let i = 0; i < a.data.length; i++) {
    if (a.data[i] != b.data[i]) return true;
  }

  return false;
}

setTimeout(() => {
  console.log('start');
  let ctx = document.querySelector('canvas').getContext('2d');
  let write = ctx.putImageData.bind(ctx);

  window.concat = [];
  let lastImage = new ImageData(1, 1);
  ctx.putImageData = (image, x, y) => {
    if (window.concat != null && imageDiff(image, lastImage)) {
      lastImage = new ImageData(new Uint8ClampedArray(image.data), image.width);
      window.concat.push(lastImage);

      console.log(window.concat.length);
      if (window.concat.length >= 250) {
        let i = 0;
        for (image of window.concat) {
          i += 1;
          let canvas = document.createElement('canvas');
          canvas.width = image.width;
          canvas.height = image.height;
          let ctx = canvas.getContext('2d');
          ctx.putImageData(image, 0, 0);

          canvas.toBlob((blob) => {
            let link = document.createElement('a');
            link.href = URL.createObjectURL(blob);
            link.download = `frame-${i}.png`;

            link.click();
            URL.revokeObjectURL(link.href);
          }, 'image/png');
        }

        window.concat = null;
      }

      write(image, x, y);
    }
  }
}, 2000);
```

</details>
