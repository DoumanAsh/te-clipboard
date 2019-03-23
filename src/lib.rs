use winapi::um::winbase::lstrlenW as wstrlen;
use clipboard_win::set_clipboard_string;

use std::os::raw::{c_char};
use std::slice;

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
pub unsafe extern "C" fn OnNewSentence(sentence: *mut u16, info: *const InfoForExtension) -> *mut u16 {
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
    let mut string = String::from_utf16_lossy(string);
    let orig_len = string.len();

    for replace in Config::get().replace.iter() {
        string = replace.pattern.replacen(&string, replace.limit, replace.replacement.as_str()).to_string();
    }

    if orig_len != string.len() {
        let _ = set_clipboard_string(&string);
    }

    sentence
}
