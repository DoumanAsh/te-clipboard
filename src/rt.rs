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

fn caclulate_hash(line: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hasher};

    let mut hasher = DefaultHasher::new();
    hasher.write(line.as_bytes());
    hasher.finish()
}

fn clean_up() {
    use std::io::{BufReader, BufRead, BufWriter, Write};
    use std::fs;
    use std::collections::HashSet;

    const SAVE_GAMES: &str = "SavedGames.txt";
    const SAVE_GAMES_NEW: &str = "SavedGames.txt.new";
    #[cfg(not(windows))]
    const NEW_LINE: &[u8] = b"\n";
    #[cfg(windows)]
    const NEW_LINE: &[u8] = b"\r\n";

    pub struct RemoveOnDrop<'a>(&'a str);
    impl<'a> Drop for RemoveOnDrop<'a> {
        fn drop(&mut self) {
            let _ = fs::remove_file(self.0);
        }
    }

    let file = match fs::File::open(SAVE_GAMES) {
        Ok(file) => file,
        _ => return,
    };

    let mut new_file = match fs::File::create(SAVE_GAMES_NEW) {
        Ok(file) => BufWriter::new(file),
        _ => return,
    };

    let _remove_new_file = RemoveOnDrop(SAVE_GAMES_NEW);

    let mut count = 0;
    let mut store = HashSet::new();

    for line in BufReader::new(file).lines() {
        let line = match line {
            Ok(line) => line,
            _ => return,
        };

        match store.insert(caclulate_hash(&line)) {
            true => match new_file.write(line.as_bytes()).and_then(|_| new_file.write(NEW_LINE)) {
                Ok(_) => (),
                _ => return,
            },
            false => {
                count += 1;
                continue;
            }
        }
    }

    drop(new_file);

    match count {
        0 => (),
        _ => match fs::rename(SAVE_GAMES_NEW, SAVE_GAMES) {
            Ok(_) => drop(_remove_new_file),
            _ => (),
        }
    };
}

fn init() -> c_int {
    //Access lazy static on load instead of during when processing text
    crate::config::Config::get();
    clean_up();

    1
}

fn terminate() -> c_int {
    1
}
