/// A named block of numbers, row by row.
#[derive(Clone, Debug, PartialEq)]
pub struct Tensor {
    pub name: String,
    pub dims: Vec<usize>,
    pub data: Vec<f64>,
}

impl Tensor {
    pub fn zeros_like(&self, name: String) -> Self {
        Self {
            name,
            dims: self.dims.clone(),
            data: vec![0.0; self.data.len()],
        }
    }

    pub fn at(&self, row: usize, col: usize) -> f64 {
        self.data[row * self.dims[1] + col]
    }

    /// `[[i64; 4]; 3]` for a 3 by 4 matrix.
    pub fn roop_type(&self) -> String {
        self.dims
            .iter()
            .rev()
            .fold("i64".to_string(), |inner, d| format!("[{inner}; {d}]"))
    }

    /// `w0[1][2]` for the element at a position in `data`.
    pub fn element(&self, flat: usize) -> String {
        let mut rest = flat;
        let mut path = Vec::new();
        for (k, _) in self.dims.iter().enumerate().rev() {
            path.push(rest % self.dims[k]);
            rest /= self.dims[k];
        }
        path.reverse();
        let indices: String = path.iter().map(|i| format!("[{i}]")).collect();
        format!("{}{indices}", self.name)
    }
}
