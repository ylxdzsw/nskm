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
//  the third argument of ioctl has the type c_ulong

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
    hook(*argv.offset(1))
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

unsafe fn hook(source: c_str) -> ! {
    let fdi = open(source, O_RDONLY);
    let fdo = open("/dev/uinput\0".as_ptr() as c_str, O_WRONLY | O_NONBLOCK);
    if fdi < 0 || fdo < 0 {
        die("cannot open input device file (hint: are you root?)");
    }

    ioctl(fdo, UI_SET_EVBIT, EV_SYN as c_ulong).dienz("set evbit"); // for absolute positioning
    ioctl(fdo, UI_SET_EVBIT, EV_KEY as c_ulong).dienz("set evbit"); // this also includes mouse buttons
    // ioctl(fdo, UI_SET_EVBIT, EV_ABS as c_ulong).dienz("set evbit"); // for absolute positioning

    // sucking loop 'cause Rust range iterations use Option internally.
    let mut i: c_ulong = 0;
    while i < KEY_MAX as _ {
        ioctl(fdo, UI_SET_KEYBIT, i).dienz("set keybit");
        i += 1;
    }

    let mut setup: uinput_setup = core::mem::zeroed();
    let name = "uinput-nskm"; // dest has been already zeroed
    memcpy(&mut setup.name as *mut _ as _, name.as_ptr() as _, name.len() as _);
    setup.id = input_id { bustype: BUS_VIRTUAL, vendor: 39, product: 39, version: 39 };
    ioctl(fdo, UI_DEV_SETUP, &setup).dienz("setup dev");
    ioctl(fdo, UI_DEV_CREATE).dienz("create dev");

    ioctl(fdi, EVIOCGRAB, 1 as c_ulong).dienz("grabbing io");

    let mut ev: input_event = core::mem::zeroed(); // could be uninitialized
    loop {
        read(fdi, &mut ev as *mut _ as _, size_of_val(&ev)).dienz("read ev");

        // main logic goes here
        if ev.code == KEY_D {
            continue
        }

        ev.time = core::mem::zeroed();

        write(fdo, &mut ev as *mut _ as _, size_of_val(&ev)).dienz("write ev");
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

impl DieNZ for ssize_t { // read/write
    unsafe fn dienz(&self, msg: &str) {
        if *self < 0 {
            die(msg)
        }
    }
}
