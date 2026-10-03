/// A parsed `"ij,jk->ik"`: the labels of each operand, and of the output.
#[derive(Debug, PartialEq, Eq)]
pub struct Spec {
    pub inputs: Vec<Vec<char>>,
    pub output: Vec<char>,
}

impl Spec {
    /// Every label once, in order of appearance in the inputs. These are the
    /// lengths of the function, one for each.
    pub fn labels(&self) -> Vec<char> {
        let mut seen = Vec::new();
        for label in self.inputs.iter().flatten() {
            if !seen.contains(label) {
                seen.push(*label);
            }
        }
        seen
    }

    /// The labels that are summed over: those not in the output.
    pub fn reduced(&self) -> Vec<char> {
        let kept = |l: &char| !self.output.contains(l);
        self.labels().into_iter().filter(kept).collect()
    }
}
