use std::{borrow::Cow, path::Path};
use dirs::home_dir;
use users::os::unix::UserExt;

pub fn expand_tilde<P: AsRef<Path> + ?Sized>(path: &'_ P) -> Cow<'_, Path> {
    let path = path.as_ref();
    if !path.to_string_lossy().starts_with("~") {
        return path.into();
    }

    let mut home = if let Some(home) = home_dir() {
        home
    } else {
        return path.into()
    };

    if path.starts_with("~/") {
        home.join(path.strip_prefix("~").unwrap()).into()
    } else if path.as_os_str().len() == 1 {
        // is a lone "~"
        home.into()
    } else {
        // is of format ~other_user/rest/of/path
        let path_str = path.to_string_lossy();
        let index = if let Some(idx) = path_str.find('/') {
            idx
        } else {
            path_str.len()
        };

        let (mut username, mut rest_of_path) = path_str.split_at(index);
        username = &username[1..];
        if index < path_str.len() {
            // remove leading slash so that .join doesn't go back to the root
            rest_of_path = &rest_of_path[1..];
        }

        if let Some(user) = users::get_user_by_name(username) {
            home = user.home_dir().into()
        } else {
            return path.into()
        }

        home.join(&rest_of_path).into()
    }
}
