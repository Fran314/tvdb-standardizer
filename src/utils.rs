pub fn usize_clamp(min: usize, val: i16, max: usize) -> usize {
    if val <= min as i16 {
        min
    } else if val >= max as i16 {
        max
    } else {
        val.try_into()
            .expect("value clamped between usizes must be valid usize")
    }
}
pub fn u8_mod(val: i16, m: u8) -> u8 {
    ((val + (m as i16)) % (m as i16))
        .try_into()
        .expect("value taken modulo u8 must be valid u8")
}

pub trait CharWise {
    fn char_len(&self) -> usize;
    fn char_insert(&mut self, idx: usize, ch: char);
    fn char_remove(&mut self, idx: usize);
    fn char_split_at(&self, idx: usize) -> (String, String, String);
}
impl CharWise for String {
    fn char_len(&self) -> usize {
        self.chars().count()
    }

    fn char_insert(&mut self, idx: usize, ch: char) {
        let mut chars = self.chars();
        let before: String = chars.by_ref().take(idx).collect();
        let after: String = chars.collect();
        *self = before + &ch.to_string() + &after;
    }

    fn char_remove(&mut self, idx: usize) {
        let mut chars = self.chars();
        let before: String = chars.by_ref().take(idx).collect();
        let _ = chars.by_ref().take(1).collect::<String>();
        let after: String = chars.collect();
        *self = before + &after;
    }

    fn char_split_at(&self, idx: usize) -> (String, String, String) {
        let mut chars = self.chars();
        let before: String = chars.by_ref().take(idx).collect();
        let ch: String = chars.by_ref().take(1).collect();
        let after: String = chars.collect();
        (before, ch, after)
    }
}
