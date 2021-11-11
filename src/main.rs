#![allow(irrefutable_let_patterns)]
#![allow(dead_code, unused_imports)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

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

mod hook;
mod codes;
use codes::*;
use core::ffi::c_void;
use core::mem::size_of_val;
use core::ptr::{null, null_mut};

type c_str = *const c_char; // Ensure they end with \0 !!

const V_KEYUP: __s32 = 0;
const V_KEYDOWN: __s32 = 1;
const V_KEYREP: __s32 = 2;

static mut STDERR: *mut FILE = 0 as _;

#[no_mangle]
unsafe extern "C" fn main(argc: c_int, argv: *const c_str) -> ! {
    STDERR = fdopen(STDERR_FILENO, "w\0".as_ptr() as _);
    if STDERR == 0 as _ {
        exit(-3);
    }
    if argc < 2 {
        die("no input device specified");
    }
    setup(*argv.offset(1))
}

#[panic_handler] // it should never panic anyway
unsafe fn panic(_info: &core::panic::PanicInfo) -> ! {
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

unsafe fn setup(source: c_str) -> ! {
    let fdi = open(source, O_RDONLY);
    let fdo = open("/dev/uinput\0".as_ptr() as c_str, O_WRONLY | O_NONBLOCK);
    if fdi < 0 || fdo < 0 {
        die("cannot open input device file (hint: are you root?)");
    }

    // setup key events, for keyboard AND mouse button
    ioctl(fdo, UI_SET_EVBIT, EV_KEY as c_int).dienz("set key evbit");
    let mut i: c_int = 0; // Rust range iterations use Option internally.
    while i < 0x223 { // systemd complains if we loop up to KEY_MAX. Didn't figure out which key is problematic.
        ioctl(fdo, UI_SET_KEYBIT, i).dienz("set keybit");
        i += 1;
    }

    // setup rel events, for relative positioning
    ioctl(fdo, UI_SET_EVBIT, EV_REL as c_int).dienz("set rel evbit");
    ioctl(fdo, UI_SET_RELBIT, REL_X as c_int).dienz("set relbit");
    ioctl(fdo, UI_SET_RELBIT, REL_Y as c_int).dienz("set relbit");

    let mut udev_setup: uinput_setup = core::mem::zeroed();
    let name = "uinput-nskm"; // dest has already been zeroed
    memcpy(&mut udev_setup.name as *mut _ as _, name.as_ptr() as _, name.len() as _);
    udev_setup.id = input_id { bustype: BUS_VIRTUAL, vendor: 39, product: 39, version: 39 };
    ioctl(fdo, UI_DEV_SETUP, &udev_setup).dienz("setup dev");
    ioctl(fdo, UI_DEV_CREATE).dienz("create dev");

    ioctl(fdi, EVIOCGRAB, 1 as c_ulong).dienz("grabbing io (hint: is NSKM already running?)");

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

        // rescue key: hold ctrl key on both sides and press K to kill the process
        static mut LCtrl: bool = false;
        static mut RCtrl: bool = false;
        if ev.type_ == EV_KEY {
            match (ev.code, ev.value) {
                (KEY_LEFTCTRL, V_KEYDOWN) => LCtrl = true,
                (KEY_LEFTCTRL, V_KEYUP) => LCtrl = false,
                (KEY_RIGHTCTRL, V_KEYDOWN) => RCtrl = true,
                (KEY_RIGHTCTRL, V_KEYUP) => RCtrl = false,
                (KEY_K, V_KEYDOWN) if LCtrl && RCtrl => {
                    fputs("Double Ctrl + K detected, exiting.\n\0".as_ptr() as _, STDERR);
                    exit(0)
                },
                _ => {}
            }
        }

        hook::hook(ev, &u);
    }
}

struct UInput {
    fd: c_int
}

/// most methods returns self for chaining
impl UInput {
    /// low level API: emit an raw event
    unsafe fn emit(&self, ev: &input_event) -> &Self {
        write(self.fd, ev as *const _ as _, size_of_val(ev)).dienz("write ev");
        self
    }

    /// low level API: emit a SYN_REPORT
    unsafe fn sync(&self) -> &Self {
        self.emit(&input_event { type_: EV_SYN, code: SYN_REPORT, ..core::mem::zeroed() })
    }

    /// move cursor by (x, y)
    unsafe fn rel(&self, x: __s32, y: __s32) -> &Self {
        if x != 0 {
            self.emit(&input_event { type_: EV_REL, code: REL_X, value: x, ..core::mem::zeroed() });
        }
        if y != 0 {
            self.emit(&input_event { type_: EV_REL, code: REL_Y, value: y, ..core::mem::zeroed() });
        }
        self.sync()
    }

    /// press down (without release) a key
    unsafe fn press(&self, key: __u16) -> &Self {
        self.emit(&input_event { type_: EV_KEY, code: key, value: V_KEYDOWN, ..core::mem::zeroed() });
        self.sync()
    }

    /// release (without press first) a key
    unsafe fn release(&self, key: __u16) -> &Self {
        self.emit(&input_event { type_: EV_KEY, code: key, value: V_KEYUP, ..core::mem::zeroed() });
        self.sync()
    }

    /// press and release a key
    unsafe fn click(&self, key: __u16) -> &Self {
        self.emit(&input_event { type_: EV_KEY, code: key, value: V_KEYDOWN, ..core::mem::zeroed() });
        self.emit(&input_event { type_: EV_KEY, code: key, value: V_KEYUP, ..core::mem::zeroed() });
        self.sync()
    }
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

impl DieNZ for ssize_t { // read/write call
    unsafe fn dienz(&self, msg: &str) {
        if *self < 0 {
            die(msg)
        }
    }
}

/// the stupid way to spawn a process and send it to init
unsafe fn spawn_orphan(p: c_str) {
    match fork() {
        -1 => die("spawn child"),
        0 => if fork() == 0 { // child, fork twice
            execlp("bash\0".as_ptr() as _, "bash\0".as_ptr() as c_str, "-c\0".as_ptr() as c_str, p).dienz("spawn grand child")
        } else {
            exit(0) // suicide to send grand child to init
        },
        pid => { waitpid(pid, null_mut(), 0); }
    }
}
