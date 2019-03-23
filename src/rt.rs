use winapi::shared::minwindef::{HINSTANCE};
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows_win::ui::msg_box::MessageBox;

use std::thread;
use std::os::raw::{c_void, c_int, c_ulong};

static mut NOTIFY_SEND: Option<crossbeam_channel::Sender<&'static str>> = None;

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: c_ulong, reserved: *mut c_void) -> c_int {
    match call_reason {
        DLL_PROCESS_ATTACH => init(),
        DLL_PROCESS_DETACH => terminate(),
        _ => 1
    }
}

pub fn notify(text: &'static str) {
    let sender = unsafe { NOTIFY_SEND.as_ref() };

    match sender {
        Some(sender) => {
            let _ = sender.send(text);
        },
        None => (),
    }
}

fn init() -> c_int {
    let (sender, recv) = crossbeam_channel::bounded(1);

    thread::spawn(move || loop {
        match recv.recv() {
            Ok(text) => {
                let _ = MessageBox::info(text).title("TE-Clipboard").show();
            },
            Err(_) => break,
        }
    });

    unsafe {
        NOTIFY_SEND = Some(sender);
    }

    crate::config::Config::get();

    1
}

fn terminate() -> c_int {
    unsafe {
        NOTIFY_SEND.take();
    }

    1
}
