use smawk_row_minima::{Matrix, MongeMatrix, SearchError, brute_force, is_monge};

#[test]
fn generated_matrices_are_monge_and_reproducible() {
    for seed in 0..10 {
        let a = MongeMatrix::seeded(7, 9, seed);
        let b = MongeMatrix::seeded(7, 9, seed);
        assert!(is_monge(&a).unwrap());
        for i in 0..7 {
            for j in 0..9 {
                assert_eq!(a.at(i, j), b.at(i, j));
            }
        }
    }
}

#[test]
fn brute_force_counts_and_edges() {
    let m = MongeMatrix::new(3, vec![0, 0, 0]);
    let result = brute_force(&m).unwrap();
    assert_eq!(result.minima, vec![0, 1, 2]);
    assert_eq!(result.metrics.evaluations, 9);
    assert_eq!(result.metrics.comparisons, 6);
    assert!(
        brute_force(&MongeMatrix::new(0, vec![]))
            .unwrap()
            .minima
            .is_empty()
    );
    assert_eq!(
        brute_force(&MongeMatrix::new(1, vec![])).unwrap_err(),
        SearchError::NoColumns
    );
    assert_eq!(
        brute_force(&MongeMatrix::new(1, vec![0, -1]))
            .unwrap()
            .minima,
        vec![0]
    );
}
