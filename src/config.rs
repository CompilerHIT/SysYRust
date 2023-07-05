static mut FILE_PATH: Option<String> = None;

pub fn set_file_path(path: &String) {
    unsafe { FILE_PATH = Some(String::from(path)) };
}

pub fn get_file_path() -> Option<String> {
    if unsafe { FILE_PATH.is_none() } {
        return None;
    } else {
        return Some(unsafe {
            let str = FILE_PATH.to_owned().unwrap();
            str
        });
    }
}
