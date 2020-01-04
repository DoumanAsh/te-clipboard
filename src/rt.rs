use winapi::shared::minwindef::{HINSTANCE};
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows_win::ui::msg_box::MessageBox;

use std::thread;
use std::os::raw::{c_void, c_int, c_ulong};
use std::sync::mpsc::{self, sync_channel};
use std::sync::Once;
use core::mem::MaybeUninit;

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
    static mut NOTIFY_SEND: MaybeUninit<mpsc::SyncSender<&'static str>> = MaybeUninit::uninit();
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let (sender, recv) = sync_channel(1);

        thread::spawn(move || loop {
            match recv.recv() {
                Ok(text) => {
                    let _ = MessageBox::info(text).title("TE-Clipboard").show();
                },
                Err(_) => break,
            }
        });

        unsafe {
            core::ptr::write(NOTIFY_SEND.as_mut_ptr(), sender);
        }
    });

    let _ = unsafe { &*NOTIFY_SEND.as_ptr() }.send(text);
}

fn init() -> c_int {
    //Access lazy static on load instead of during when processing text
    crate::config::Config::get();

    1
}

fn terminate() -> c_int {
    1
}
