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
    // All 3^6 two-row, three-column matrices, including ties and non-Monge inputs.
    for mut code in 0..729 {
        let mut cells = vec![vec![0; 3]; 2];
        for row in &mut cells {
            for value in row {
                *value = code % 3;
                code /= 3;
            }
        }
        let tm = (0..3)
            .all(|a| (a + 1..3).all(|b| cells[0][a] <= cells[0][b] || cells[1][a] > cells[1][b]));
        if tm {
            check(&Dense(cells));
        }
    }
}

#[test]
fn evaluation_and_space_growth_are_linear() {
    for (rows, cols) in [(64, 64), (512, 512), (1024, 17), (17, 1024)] {
        let result = smawk(&MongeMatrix::seeded(rows, cols, 42)).unwrap();
        assert!(result.metrics.evaluations <= 12 * (rows + cols));
        assert!(result.metrics.buffer_bytes <= 12 * (rows + cols) * size_of::<usize>());
    }
}
