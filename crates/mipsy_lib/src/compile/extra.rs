use crate::Binary;

pub(super) fn move_labels<'a>(
    binary: &mut Binary,
    moves: impl Iterator<Item = (&'a str, &'a str)>,
) {
    for (old, new) in moves {
        let new_addr = binary
            .get_label(new)
            .unwrap_or_else(|_| panic!("move-label used with non-existent label {new}"));

        binary.insert_label(old, new_addr);
    }
}
