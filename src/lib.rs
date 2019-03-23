use winapi::um::winbase::lstrlenW as wstrlen;

use std::os::raw::{c_void, c_char};
use std::borrow::Cow;
use std::os::windows::ffi::OsStrExt;
use std::{slice, ptr, mem};

mod config;
mod rt;

use config::Config;

#[repr(C)]
pub struct InfoForExtension {
    name: *const c_char,
    value: i64
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn OnNewSentence(mut sentence: *mut u16, info: *const InfoForExtension) -> *mut u16 {
    if info.is_null() {
        return sentence;
    }

    //Depends on order https://github.com/Artikash/Textractor/blob/21fd3e1d5932b1bd65ef23eb59dd6a49a35ff22f/GUI/mainwindow.cpp#L216
    let info = slice::from_raw_parts(info, 10);
    if info[0].value == 0 || info[2].value == 0 {
        return sentence;
    }

    let size = match wstrlen(sentence) {
        0 | 1 => return sentence,
        size => size,
    };

    let string = slice::from_raw_parts(sentence, size as usize);
    let string = String::from_utf16_lossy(string);
    let original_len = string.len();
    let mut string = Cow::Owned(string);

    let config = Config::get();
    for replace in config.replace.iter() {
        match replace.pattern.replacen(&string, replace.limit, replace.replacement.as_str()) {
            Cow::Owned(new) => {
                string = Cow::Owned(new);
            },
            _ => (),
        }
    }

    let data = std::ffi::OsStr::new(string.as_ref());
    let mut data = data.encode_wide().collect::<Vec<u16>>();
    data.push(0);
    let data = slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * mem::size_of::<u16>());

    let _ = clipboard_win::raw::set(clipboard_win::formats::CF_UNICODETEXT, data);

    if config.settings.modify_original {
        if original_len != string.as_ref().len() {
            let new_sentence = winapi::um::heapapi::HeapReAlloc(winapi::um::heapapi::GetProcessHeap(), 0, sentence as *mut c_void, data.len());
            if new_sentence.is_null() {
                //We back down, if allocation fails, which is unlikely, right?
                return sentence;
            } else {
                sentence = new_sentence as *mut u16;
            }
        }

        ptr::copy_nonoverlapping(data.as_ptr(), sentence as *mut u8, data.len());
    }

    sentence
}
