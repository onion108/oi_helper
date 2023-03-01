use std::{fs::{self, OpenOptions}, path::PathBuf, io::Write};

use localoco::buildscript::build_json;

fn compile_lang(homedir: &PathBuf, name: &str) {
    match build_json(&format!("resources/lang/{name}.json")) {
        Ok(bytes) => {
            let mut c = homedir.clone();
            c.push("lang");
            if !c.as_path().exists() {
                fs::create_dir(&c).unwrap();
            }
            c.push(&format!("{name}.loc"));
            let mut f = OpenOptions::new().write(true).create(true).open(c).unwrap();
            f.write_all(&bytes).unwrap();
        }
        Err(err) => {
            println!("Error building: {}", err)
        }
    }
}

fn main() {
    let homedir;
    if let Some(user_home) = home::home_dir() {
        let mut p = user_home.clone();
        p.push(".oi_helper");
        if !p.as_path().exists() {
            fs::create_dir(&p).unwrap();
        }
        homedir = p;
    } else {
        return;
    }

    // Compile language files and move them into the target directory.
    compile_lang(&homedir, "en_US");
    compile_lang(&homedir, "zh_CN");
}
