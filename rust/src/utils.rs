use std::path::PathBuf;

use godot::engine::file_access::ModeFlags;
use godot::engine::utilities::error_string;
use godot::engine::FileAccess;
use godot::{engine::DirAccess, prelude::*};
use godot::engine::global::Error as GdErr;

pub fn get_files_recursive(path: GString) -> Vec<GString> {
    let da = DirAccess::open(path.clone());
    if da.is_none() || DirAccess::get_open_error() != GdErr::OK {
        godot_error!("Unable to recurse through folder {}: {}", path, error_string(DirAccess::get_open_error().ord() as i64));
        return vec![];
    }
    let mut da = da.unwrap();
    if da.list_dir_begin() != GdErr::OK {
        return vec![];
    }

    let mut files = vec![];
    let mut file_name = da.get_next();
    while !file_name.is_empty() {
        if da.current_is_dir() {
            if &file_name.to_string() != ".godot" {
                // TODO: use GString.path_join() once available (upstream issue)
                let child_path = format!("{}/{}", path.clone(), file_name.clone()).into();
                let mut recurse = get_files_recursive(child_path);
                files.append(&mut recurse);
            }
        } else {
            files.push(file_name);
        }
        file_name = da.get_next();
    }
    files
}

pub fn create_or_open_file_for_read_write(path: GString) -> Result<Gd<FileAccess>, GdErr> {
    let dir_only = match PathBuf::from(path.clone().to_string()).parent() {
        Some(dir) => dir.to_str().unwrap_or_default().into(),
        None => String::new(),
    };
    let dir_err = DirAccess::make_dir_recursive_absolute(dir_only.into());
    if dir_err != GdErr::OK {
        return Err(dir_err);
    }

    let fa = FileAccess::open(path.clone(), ModeFlags::READ_WRITE);
    if fa.is_none() || FileAccess::get_open_error() != GdErr::OK {
        if FileAccess::get_open_error() == GdErr::ERR_FILE_NOT_FOUND {
            let fa = FileAccess::open(path.clone(), ModeFlags::WRITE_READ);
            if fa.is_some() && FileAccess::get_open_error() == GdErr::OK {
                return Ok(fa.unwrap());
            }
        }
        return Err(FileAccess::get_open_error());
    }
    Ok(fa.unwrap())
}
