/// Values must be deterministic and evaluable in constant time for SMAWK's bound.
pub trait Matrix {
    fn rows(&self) -> usize;
    fn cols(&self) -> usize;
    fn at(&self, row: usize, col: usize) -> i128;
}

/// Implicit Monge matrix `(row - col)^2 + cost[col]`, storing only column costs.
pub struct MongeMatrix {
    rows: usize,
    costs: Vec<i64>,
}

impl MongeMatrix {
    pub fn new(rows: usize, costs: Vec<i64>) -> Self {
        Self { rows, costs }
    }

    /// SplitMix64 with wrapping arithmetic; costs are in -100..=100.
    pub fn seeded(rows: usize, cols: usize, seed: u64) -> Self {
        let mut state = seed;
        let costs = (0..cols)
            .map(|_| {
                state = state.wrapping_add(0x9e3779b97f4a7c15);
                let mut z = state;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
                ((z ^ (z >> 31)) % 201) as i64 - 100
            })
            .collect();
        Self::new(rows, costs)
    }
}

impl Matrix for MongeMatrix {
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.costs.len()
    }
    fn at(&self, row: usize, col: usize) -> i128 {
        assert!(row < self.rows);
        let delta = row as i128 - col as i128;
        delta
            .checked_mul(delta)
            .and_then(|v| v.checked_add(self.costs[col] as i128))
            .expect("matrix value exceeds i128")
    }
}

/// Exhaustive rectangle check, O(rows^2 * cols^2). Returns an error on sum overflow.
pub fn is_monge<M: Matrix>(matrix: &M) -> Result<bool, &'static str> {
    for a in 0..matrix.rows() {
        for b in a + 1..matrix.rows() {
            for c in 0..matrix.cols() {
                for d in c + 1..matrix.cols() {
                    let left = matrix
                        .at(a, c)
                        .checked_add(matrix.at(b, d))
                        .ok_or("Monge sum overflow")?;
                    let right = matrix
                        .at(a, d)
                        .checked_add(matrix.at(b, c))
                        .ok_or("Monge sum overflow")?;
                    if left > right {
                        return Ok(false);
                    }
                }
            }
        }
    }
    Ok(true)
}
