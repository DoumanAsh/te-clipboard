use winapi::shared::minwindef::{HINSTANCE};
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows_win::ui::msg_box::MessageBox;
use lazy_static::lazy_static;

use std::thread;
use std::os::raw::{c_void, c_int, c_ulong};
use std::sync::mpsc::{self, sync_channel};

lazy_static! {
    static ref NOTIFY_SEND: mpsc::SyncSender<&'static str> = {
        let (sender, recv) = sync_channel(1);

        thread::spawn(move || loop {
            match recv.recv() {
                Ok(text) => {
                    let _ = MessageBox::info(text).title("TE-Clipboard").show();
                },
                Err(_) => break,
            }
        });

        sender
    };
}

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
    let _ = NOTIFY_SEND.send(text);
}

fn init() -> c_int {
    //Access lazy static on load instead of during when processing text
    crate::config::Config::get();

    1
}

fn terminate() -> c_int {
    1
}
