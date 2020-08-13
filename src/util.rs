pub fn ok<T: Default, E>() -> Result<T, E> {
    Ok(T::default())
}
