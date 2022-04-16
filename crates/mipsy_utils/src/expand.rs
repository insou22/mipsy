use std::{borrow::Cow, path::Path};
use dirs::home_dir;

pub fn expand_tilde<P: AsRef<Path> + ?Sized>(path: &'_ P) -> Cow<'_, Path> {
    let path = path.as_ref();
    if !path.starts_with("~") {
        return path.into();
    }

    // TODO: support ~other_user/rest/of/path
    if let Some(home) = home_dir() {
        home.join(path.strip_prefix("~").unwrap()).into()
    } else {
        path.into()
    }
}
