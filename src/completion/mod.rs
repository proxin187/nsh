use std::fs;
use std::env;


fn as_local(global: &str) -> Result<String, Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let mut local = global.to_string();

    for _ in 0..current_dir.to_str().expect("str convert failed").len() + 1 {
        local.remove(0);
    }

    Ok(local)
}


pub fn complete(original_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    for dir in fs::read_dir(current_dir)? {
        let path = dir?.path();

        let path_str = as_local(path
                                .as_os_str()
                                .to_str()
                                .unwrap())?;

        if path_str.starts_with(original_path) {
            return Ok(path_str);
        }
    }

    Ok(original_path.to_string())
}


