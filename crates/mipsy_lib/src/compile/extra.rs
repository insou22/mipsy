use crate::Binary;

pub(super) fn move_labels<'a>(binary: &mut Binary, moves: impl Iterator<Item = (&'a str, &'a str)>) {
    for (old, new) in moves {
        let new_addr = binary.get_label(new)
            .expect(&format!("move-label used with non-existent label {new}"));
        
        binary.insert_label(old, new_addr);
    }
}
