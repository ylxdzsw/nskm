#![allow(irrefutable_let_patterns)]
#![allow(dead_code, unused_imports)]
#![allow(non_camel_case_types)]
#![deny(bare_trait_objects)]
#![warn(clippy::all)]

#![no_std]
#![no_main]

// Exit codes:
//   0: success
//  -1: failure caused by bad arguments or system call
//  -2: errors that should not happen (bug)
//  -3: we can't even open stderr so we can't leave an error message

// Notes:
//  the third argument of ioctl is either int or pointer

use libc::*;

mod codes;
use codes::*;
use core::ffi::c_void;
use core::mem::size_of_val;

type c_str = *const c_char; // Ensure they end with \0 !!

static mut STDERR: *mut FILE = 0 as _;

#[no_mangle]
unsafe extern "C" fn main(argc: c_int, argv: *const c_str) -> ! {
    STDERR = fdopen(STDERR_FILENO, "w".as_ptr() as _);
    if STDERR == 0 as _ {
        exit(-3);
    }
    if argc < 2 {
        die("no input device specified");
    }
    setup(*argv.offset(1))
}

#[panic_handler]
unsafe fn panic(_info: &core::panic::PanicInfo) -> ! {
    // it should never panic
    exit(-2);
}

/// print error message and stop running. SHOULD NOT catch this because we weren't clean before dying
unsafe fn die(msg: &str) -> ! {
    fprintf(STDERR, "error: %.*s\0".as_ptr() as _, msg.len() as c_int, msg.as_ptr());
    exit(-1)
}

#[repr(C)]
pub struct uinput_setup {
	pub id: input_id,
	pub name: [c_char; UINPUT_MAX_NAME_SIZE],
	pub ff_effects_max: __u32,
}

#[repr(C)]
pub struct uinput_abs_setup {
	pub code: __u16,
    pub absinfo: input_absinfo
}

unsafe fn setup(source: c_str) -> ! {
    let fdi = open(source, O_RDONLY);
    let fdo = open("/dev/uinput\0".as_ptr() as c_str, O_WRONLY | O_NONBLOCK);
    if fdi < 0 || fdo < 0 {
        die("cannot open input device file (hint: are you root?)");
    }

    // setup key events, for keyboard AND mouse button
    ioctl(fdo, UI_SET_EVBIT, EV_KEY as c_int).dienz("set key evbit");
    let mut i: c_int = 0; // sucking loop 'cause Rust range iterations use Option internally.
    while i < KEY_MAX as _ {
        ioctl(fdo, UI_SET_KEYBIT, i).dienz("set keybit");
        i += 1;
    }

    // setup rel events, for relative positioning
    ioctl(fdo, UI_SET_EVBIT, EV_REL as c_int).dienz("set rel evbit");
    ioctl(fdo, UI_SET_RELBIT, REL_X as c_int).dienz("set relbit");
    ioctl(fdo, UI_SET_RELBIT, REL_Y as c_int).dienz("set relbit");

    let mut udev_setup: uinput_setup = core::mem::zeroed();
    let name = "uinput-nskm"; // dest has been already zeroed
    memcpy(&mut udev_setup.name as *mut _ as _, name.as_ptr() as _, name.len() as _);
    udev_setup.id = input_id { bustype: BUS_VIRTUAL, vendor: 39, product: 39, version: 39 };
    ioctl(fdo, UI_DEV_SETUP, &udev_setup).dienz("setup dev");
    ioctl(fdo, UI_DEV_CREATE).dienz("create dev");

    ioctl(fdi, EVIOCGRAB, 1 as c_ulong).dienz("grabbing io");

    let mut ev: input_event = core::mem::zeroed(); // could be uninitialized
    let mut syn_dropped = false; // indicating overrun in evdev. Should drop all events up to and including next SYN_REPORT
    let u = UInput { fd: fdo };

    loop {
        read(fdi, &mut ev as *mut _ as _, size_of_val(&ev)).dienz("read ev");

        if syn_dropped {
            if ev.type_ == EV_SYN && ev.code == SYN_REPORT {
                syn_dropped = false;
            }
            continue
        } else if ev.type_ == EV_SYN && ev.code == SYN_DROPPED {
            syn_dropped = true;
            continue
        }

        fuck(&ev, &u);
    }
}

struct UInput {
    fd: c_int
}

impl UInput {
    /// low level API: emit an raw event
    unsafe fn emit(&self, ev: &input_event) {
        write(self.fd, ev as *const _ as _, size_of_val(ev)).dienz("write ev")
    }

    /// low level API: emit a SYN_REPORT
    unsafe fn sync(&self) {
        self.emit(&input_event { type_: EV_SYN, code: SYN_REPORT, ..core::mem::zeroed() })
    }

    /// move cursor by (x, y)
    unsafe fn rel(&self, x: __s32, y: __s32) {
        if x != 0 {
            self.emit(&input_event { type_: EV_REL, code: REL_X, value: x, ..core::mem::zeroed() })
        }
        if y != 0 {
            self.emit(&input_event { type_: EV_REL, code: REL_Y, value: y, ..core::mem::zeroed() })
        }
        self.sync()
    }

    /// press down (without release) a key
    unsafe fn press(&self, key: __u16) {
        self.emit(&input_event { type_: EV_KEY, code: key, value: 1, ..core::mem::zeroed() });
        self.sync();
    }

    /// release (without press first) a key
    unsafe fn release(&self, key: __u16) {
        self.emit(&input_event { type_: EV_KEY, code: key, value: 0, ..core::mem::zeroed() });
        self.sync()
    }

    /// press and release a key
    unsafe fn click(&self, key: __u16) {
        self.emit(&input_event { type_: EV_KEY, code: key, value: 0, ..core::mem::zeroed() });
        self.emit(&input_event { type_: EV_KEY, code: key, value: 1, ..core::mem::zeroed() });
        self.sync()
    }
}

unsafe fn fuck(ev: &input_event, u: &UInput) {
    static mut x: __s32 = 2;
    if ev.type_ == EV_KEY && ev.code == KEY_D {
        u.rel(x, 4);
        x += 1;
        return
    }
    u.emit(ev);
    u.sync()
}

trait DieNZ {
    unsafe fn dienz(&self, msg: &str);
}

impl DieNZ for c_int { // ioctl
    unsafe fn dienz(&self, msg: &str) {
        if *self < 0 {
            die(msg)
        }
    }
}

impl DieNZ for ssize_t { // read/write
    unsafe fn dienz(&self, msg: &str) {
        if *self < 0 {
            die(msg)
        }
    }
}
