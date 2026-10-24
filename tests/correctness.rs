use smawk_row_minima::{Matrix, MongeMatrix, brute_force, is_monge, smawk, smawk_traced};

struct Dense(Vec<Vec<i128>>);
impl Matrix for Dense {
    fn rows(&self) -> usize {
        self.0.len()
    }
    fn cols(&self) -> usize {
        self.0.first().map_or(0, Vec::len)
    }
    fn at(&self, r: usize, c: usize) -> i128 {
        self.0[r][c]
    }
}

fn check(matrix: &impl Matrix) {
    let expected = brute_force(matrix).unwrap();
    let actual = smawk(matrix).unwrap();
    assert_eq!(actual.minima, expected.minima);
    let (traced, trace) = smawk_traced(matrix).unwrap();
    assert_eq!(traced.minima, expected.minima);
    assert_eq!(traced.metrics.evaluations, actual.metrics.evaluations);
    assert_eq!(
        trace
            .events
            .iter()
            .filter(|e| matches!(e, smawk_row_minima::Event::Evaluate { .. }))
            .count(),
        actual.metrics.evaluations
    );
}

#[test]
fn rectangular_seed_sweep() {
    for rows in 0..25 {
        for cols in 1..25 {
            for seed in 0..12 {
                check(&MongeMatrix::seeded(rows, cols, seed));
            }
        }
    }
    for (rows, cols) in [(1, 4096), (4096, 1), (3, 4096), (4096, 3)] {
        check(&MongeMatrix::seeded(rows, cols, 42));
    }
}

#[test]
fn ties_boundaries_and_non_monge() {
    for matrix in [
        Dense(vec![vec![7; 8]; 9]),
        Dense(vec![vec![0, 1, 2]; 5]),
        Dense(vec![vec![2, 1, 0]; 5]),
        Dense(vec![vec![0, 1], vec![0, 2]]),
        Dense(vec![vec![1, 1, 2], vec![2, 1, 1], vec![3, 2, 1]]),
    ] {
        check(&matrix);
    }
    assert!(!is_monge(&Dense(vec![vec![0, 1], vec![0, 2]])).unwrap());
    assert!(
        smawk(&MongeMatrix::new(0, vec![]))
            .unwrap()
            .minima
            .is_empty()
    );
    assert!(smawk(&MongeMatrix::new(1, vec![])).is_err());
}

#[test]
fn exhaustive_small_totally_monotone_matrices() {
    // Enumerate all ternary matrices up to 3x3, then independently filter
    // using the defining implication for every pair of rows and columns.
    for rows in 1..=3 {
        for mut code in 0..3_i128.pow(rows as u32 * 3) {
            let mut cells = vec![vec![0; 3]; rows];
            for row in &mut cells {
                for value in row {
                    *value = code % 3;
                    code /= 3;
                }
            }
            let tm = (0..rows).all(|r| {
                (r + 1..rows).all(|s| {
                    (0..3).all(|a| {
                        (a + 1..3).all(|b| cells[r][a] <= cells[r][b] || cells[s][a] > cells[s][b])
                    })
                })
            });
            if tm {
                check(&Dense(cells));
            }
        }
    }
}

#[test]
fn reported_evaluations_match_actual_oracle_calls() {
    struct Counted {
        matrix: MongeMatrix,
        calls: std::cell::Cell<usize>,
    }
    impl Matrix for Counted {
        fn rows(&self) -> usize {
            self.matrix.rows()
        }
        fn cols(&self) -> usize {
            self.matrix.cols()
        }
        fn at(&self, r: usize, c: usize) -> i128 {
            self.calls.set(self.calls.get() + 1);
            self.matrix.at(r, c)
        }
    }
    let matrix = Counted {
        matrix: MongeMatrix::seeded(31, 53, 42),
        calls: std::cell::Cell::new(0),
    };
    let fast = smawk(&matrix).unwrap();
    assert_eq!(fast.metrics.evaluations, matrix.calls.replace(0));
    let brute = brute_force(&matrix).unwrap();
    assert_eq!(brute.metrics.evaluations, matrix.calls.get());
    assert_eq!(fast.minima, brute.minima);
}

#[test]
fn monge_check_reports_overflow() {
    assert!(is_monge(&Dense(vec![vec![i128::MAX; 2]; 2])).is_err());
}

#[test]
fn evaluation_and_space_growth_are_linear() {
    for (rows, cols) in [(64, 64), (512, 512), (1024, 17), (17, 1024)] {
        let result = smawk(&MongeMatrix::seeded(rows, cols, 42)).unwrap();
        assert!(result.metrics.evaluations <= 12 * (rows + cols));
        assert!(result.metrics.buffer_bytes <= 12 * (rows + cols) * size_of::<usize>());
    }
}
