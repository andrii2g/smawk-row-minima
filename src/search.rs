use crate::Matrix;
use std::time::{Duration, Instant};

#[derive(Debug, Default, Clone)]
pub struct Metrics {
    pub evaluations: usize,
    pub comparisons: usize,
    /// Candidate removals summed across recursive subproblems.
    pub eliminated_columns: usize,
    /// Root counts as depth 1; empty searches have depth 0.
    pub recursion_depth: usize,
    pub elapsed: Duration,
    /// Peak live algorithm vector capacity, including output; excludes input and trace.
    pub buffer_bytes: usize,
}

#[derive(Debug)]
pub struct SearchResult {
    pub minima: Vec<usize>,
    pub metrics: Metrics,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SearchError {
    NoColumns,
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a matrix with rows must have at least one column")
    }
}
impl std::error::Error for SearchError {}

pub fn brute_force<M: Matrix>(matrix: &M) -> Result<SearchResult, SearchError> {
    if matrix.rows() > 0 && matrix.cols() == 0 {
        return Err(SearchError::NoColumns);
    }
    let start = Instant::now();
    let mut metrics = Metrics::default();
    let mut minima = Vec::with_capacity(matrix.rows());
    for row in 0..matrix.rows() {
        let mut best = 0;
        let mut value = matrix.at(row, best);
        metrics.evaluations += 1;
        for col in 1..matrix.cols() {
            let candidate = matrix.at(row, col);
            metrics.evaluations += 1;
            metrics.comparisons += 1;
            if candidate < value {
                best = col;
                value = candidate;
            }
        }
        minima.push(best);
    }
    metrics.buffer_bytes = minima.capacity() * std::mem::size_of::<usize>();
    metrics.elapsed = start.elapsed();
    Ok(SearchResult { minima, metrics })
}
