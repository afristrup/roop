/// A test fixture as the C driver sees it: what it is made of and how much.
pub struct Fixture {
    pub name: String,
    /// `i` for 64-bit integers, `f` for doubles, `b` for bools, `u` for bytes, `e` for enums.
    pub kind: char,
    /// How many of that kind, counting the length of a stack.
    pub count: usize,
}
