use std::{borrow::Cow, path::Path};
use dirs::home_dir;

pub fn expand_tilde<P: AsRef<Path> + ?Sized>(path: &'_ P) -> Cow<'_, Path> {
    let path = path.as_ref();
    if !path.starts_with("~") {
        return path.into();
    }

    let home = if let Some(home) = home_dir() {
        home
    } else {
        return path.into()
    };

    if path.starts_with("~/") {
        home.join(path.strip_prefix("~").unwrap()).into()
    } else {
        // TODO: support ~other_user/rest/of/path
        // for now, assume this is a lone ~
        home.into()
    }
}
