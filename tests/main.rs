use std::fs;
use std::os::windows::fs::MetadataExt;


fn is_hidden(file_path: &str) -> bool {
    let metadata = match fs::metadata(file_path) {
        Ok(x) => x,
        Err(_) => {
            println!("error getting metadata for : {file_path}");
            return false;
        }, // dont show error files
    };
    let attributes = metadata.file_attributes();

    if (attributes & 0x2) > 0 {
        true
    } else {
        false
    }
}


fn main() {
    println!("{}", is_hidden("C:/users/aaron/.cargo"));
}
