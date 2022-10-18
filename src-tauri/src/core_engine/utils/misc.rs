pub fn get_index_of_first_difference(s1: &String, s2: &String) -> Option<usize> {
    let iter = s1.chars().enumerate().zip(s2.chars());
    for ((i, c1), c2) in iter {
        if c2 != c1 {
            return Some(i);
        }
    }
    None
}
