use std::{borrow::Cow, path::Path};

#[cfg(unix)]
use {dirs::home_dir, std::path::PathBuf, users::os::unix::UserExt};

#[cfg(not(unix))]
pub fn expand_tilde<P>(path: &'_ P) -> Cow<'_, Path>
where
    P: AsRef<Path> + ?Sized,
{
    // don't bother expanding on windows systems
    path.as_ref().into()
}

#[cfg(unix)]
pub fn expand_tilde<P>(path: &'_ P) -> Cow<'_, Path>
where
    P: AsRef<Path> + ?Sized,
{
    let path = path.as_ref();
    let path_str = path.to_string_lossy();
    if !path_str.starts_with('~') {
        return path.into();
    }

    let mut home = if let Some(home) = home_dir() {
        home
    } else {
        return path.into();
    };

    if path.starts_with("~/") {
        home.join(path.strip_prefix("~").unwrap()).into()
    } else if path.as_os_str().len() == 1 {
        // is a lone "~"
        home.into()
    } else {
        // is of format ~other_user/rest/of/path
        let index = if let Some(idx) = path_str.find('/') {
            idx
        } else {
            path_str.len()
        };

        let username = &path_str[1..index];
        if let Some(user) = users::get_user_by_name(username) {
            home = user.home_dir().into();
        } else {
            return path.into();
        }

        home.join(path.components().skip(1).collect::<PathBuf>())
            .into()
    }
}
