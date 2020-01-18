NSKM
====

The non-sucking key mapper that works on Wayland, by hooking into `/dev/input`.

### Features

- Works for X11, Wayland, and TTY.
- **No memory leak** because there is no heap allocation at the first place.
- **No zombies** as `nskm` gifts all children to `init` immediately after spawning.  

### Dependency

NSKM is built on top of the [uinput kernel module](https://www.kernel.org/doc/html/v5.4/input/uinput.html). If you are
using Ubuntu, great, you already have it. Otherwise, you probably know how to get and load it.

Another thing is a rust compiler. It should be packaged for most distros, but installing via
[rustup](https://www.rust-lang.org/tools/install#rustup) is also just a one-liner.

### About the magic numbers

`src/codes.rs` contains some constants found in Linux x64 5.4.2 headers, but I'm not sure how stable they are. If by any
chance those magic numbers changed, the file can be regenerated by:

```sh
$ cc sys/codegen.c && ./a.out > src/codes.rs
```

### Messed things up and unable to Ctrl + C?

Don't panic, there is a hard coded <kbd>LeftCtrl</kbd>+<kbd>RightCtrl</kbd>+<kbd>K</kbd> to kill NSKM.

## Guide

Feature rich key mappers usually come with a complicated configuration or sophisticated scripting language, which I would
rather not to touch. Thus, the designated way to use NSKM is *use the source*. Don't worry, you don't need to learn Rust
to just swap <kbd>Ctrl</kbd> and <kbd>Alt</kbd>. The guide and examples below should cover most common usages and a
copy-pasting should solve most problems.

We will use key names like `KEY_D`, `KEY_PAGEUP` in this guide. They come from `/usr/include/linux/input-event-codes.h`.
[evtest](https://gitlab.freedesktop.org/libevdev/evtest) is a great tool to help find out the code and name of a key.

### Hello World 

Let's first get a trivial mapper run: a mapper that do nothing but forward all keys. Download this project with git or
whatever, then rewrite `src/hook.rs` with the following snippet:

```rust
use super::*;

pub(crate) unsafe fn hook(ev: &input_event, u: &UInput) {
    u.emit(ev)
}
``` 

compile it with

```sh
$ cargo build
```

If no error occurs, there should be a `./target/debug/nskm` that is ready to use. Run it with 

```sh
$ sudo ./target/debug/nskm /dev/input/by-path/platform-i8042-serio-0-event-kbd
```

...wait! What is `/dev/input/by-path/platform-i8042-serio-0-event-kbd`? Well, it's the input device you want to capture,
i.e. your keyboard. Its name is likely to be exactly this, but not guaranteed to be so. `cat /proc/bus/input/devices` might
help if your keyboard is not named like this. This file is usually owned by `root` and no permission for others, that's
why we need `sudo` here. 

If you managed to run it, you will find, umm, nothing happened. That's because we just forwarded all key events. To be
sure that we actually succeed, modify the file as following:

```rust
use super::*;

pub(crate) unsafe fn hook(ev: &input_event, u: &UInput) {
    if ev.type_ == EV_KEY && ev.code == KEY_H {
        if ev.value == V_KEYDOWN {
            u.click(KEY_H);
            u.click(KEY_E);
            u.click(KEY_L);
            u.click(KEY_L);
            u.click(KEY_O);
            u.click(KEY_SPACE);
            u.click(KEY_W);
            u.click(KEY_O);
            u.click(KEY_R);
            u.click(KEY_L);
            u.click(KEY_D);
        }    
    } else {
        u.emit(ev);
    }
}
``` 

Compile and run again. If things are still on track, press <kbd>h</kbd> will write "hello world" now. Hooray! 
