use crate::{Event, Matrix, Metrics, SearchError, SearchResult, Trace};
use std::{mem::size_of, time::Instant};

/// O(m+n) evaluations and auxiliary space. Input must be totally monotone for
/// leftmost row minima. No property check is performed by this function.
pub fn smawk<M: Matrix>(matrix: &M) -> Result<SearchResult, SearchError> {
    run(matrix, None)
}

/// Same algorithm, recording an explanation. Elapsed time includes tracing.
pub fn smawk_traced<M: Matrix>(matrix: &M) -> Result<(SearchResult, Trace), SearchError> {
    let mut trace = Trace::default();
    let result = run(matrix, Some(&mut trace))?;
    Ok((result, trace))
}

struct Search<'a, M> {
    matrix: &'a M,
    metrics: Metrics,
    trace: Option<&'a mut Trace>,
    minima: Vec<usize>,
}

fn run<M: Matrix>(matrix: &M, trace: Option<&mut Trace>) -> Result<SearchResult, SearchError> {
    if matrix.rows() > 0 && matrix.cols() == 0 {
        return Err(SearchError::NoColumns);
    }
    let start = Instant::now();
    let mut search = Search {
        matrix,
        metrics: Metrics::default(),
        trace,
        minima: vec![0; matrix.rows()],
    };
    if matrix.rows() > 0 {
        let rows: Vec<_> = (0..matrix.rows()).collect();
        let columns: Vec<_> = (0..matrix.cols()).collect();
        let bytes =
            (rows.capacity() + columns.capacity() + search.minima.capacity()) * size_of::<usize>();
        search.solve(&rows, &columns, 1, bytes);
    }
    search.metrics.elapsed = start.elapsed();
    Ok(SearchResult {
        minima: search.minima,
        metrics: search.metrics,
    })
}

impl<M: Matrix> Search<'_, M> {
    fn emit(&mut self, event: Event) {
        if let Some(trace) = self.trace.as_mut() {
            trace.events.push(event);
        }
    }

    fn value(&mut self, depth: usize, row: usize, col: usize) -> i128 {
        self.metrics.evaluations += 1;
        let value = self.matrix.at(row, col);
        self.emit(Event::Evaluate {
            depth,
            row,
            col,
            value,
        });
        value
    }

    fn solve(&mut self, rows: &[usize], columns: &[usize], depth: usize, live_bytes: usize) {
        self.metrics.recursion_depth = self.metrics.recursion_depth.max(depth);
        let mut reduced = Vec::with_capacity(rows.len().min(columns.len()));
        for &col in columns {
            while let Some(&previous) = reduced.last() {
                let row = rows[reduced.len() - 1];
                let new = self.value(depth, row, col);
                let old = self.value(depth, row, previous);
                self.metrics.comparisons += 1;
                // Equality preserves the earlier column and therefore leftmost ties.
                if new >= old {
                    break;
                }
                reduced.pop();
                self.metrics.eliminated_columns += 1;
                self.emit(Event::Eliminate {
                    depth,
                    col: previous,
                });
            }
            if reduced.len() < rows.len() {
                reduced.push(col);
            } else {
                self.metrics.eliminated_columns += 1;
                self.emit(Event::Eliminate { depth, col });
            }
        }
        let odd: Vec<_> = rows.iter().skip(1).step_by(2).copied().collect();
        let live_bytes = live_bytes + (reduced.capacity() + odd.capacity()) * size_of::<usize>();
        self.metrics.buffer_bytes = self.metrics.buffer_bytes.max(live_bytes);
        if self.trace.is_some() {
            self.emit(Event::Level {
                depth,
                rows: rows.to_vec(),
                columns: reduced.clone(),
                recursive_rows: odd.clone(),
            });
        }
        if !odd.is_empty() {
            self.solve(&odd, &reduced, depth + 1, live_bytes);
        }

        // The cursor never moves backwards: bound lookup and scans are linear
        // in this level's candidate columns plus number of rows.
        let mut left = 0;
        for position in (0..rows.len()).step_by(2) {
            let row = rows[position];
            let mut right = reduced.len() - 1;
            if position + 1 < rows.len() {
                right = left;
                while reduced[right] != self.minima[rows[position + 1]] {
                    right += 1;
                }
            }
            self.emit(Event::Interpolate {
                depth,
                row,
                left: reduced[left],
                right: reduced[right],
            });
            let mut best = reduced[left];
            let mut best_value = self.value(depth, row, best);
            for &col in &reduced[left + 1..=right] {
                let value = self.value(depth, row, col);
                self.metrics.comparisons += 1;
                if value < best_value {
                    best = col;
                    best_value = value;
                }
            }
            self.minima[row] = best;
            self.emit(Event::Minimum {
                depth,
                row,
                col: best,
            });
            left = right;
        }
    }
}
