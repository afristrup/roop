use roop_syntax::Place;

/// One read or write a loop iteration performs, after resolving borrow
/// aliases. `local` accesses touch iteration-private ancillas.
pub struct Access {
    pub place: Place,
    pub write: bool,
    pub local: bool,
}
