//! Position-balanced cyclic blocks for future paired measurement campaigns.

pub const METHOD: &str = "randomized-cyclic-blocks-v1";

fn index(rng: &mut u64, count: usize) -> usize {
    let count = count as u64;
    let threshold = count.wrapping_neg() % count;
    loop {
        let value = crate::stats::random(rng);
        if value >= threshold {
            return (value % count) as usize;
        }
    }
}

fn shuffle(values: &mut [usize], rng: &mut u64) {
    for last in (1..values.len()).rev() {
        let other = index(rng, last + 1);
        values.swap(last, other);
    }
}

/// Each complete block visits every cyclic rotation of a shuffled lane list.
pub fn rounds(lanes: usize, count: usize, rng: &mut u64) -> Vec<Vec<usize>> {
    assert!((2..=8).contains(&lanes), "validated lane count required");
    let mut result = Vec::with_capacity(count);
    while result.len() < count {
        let mut labels: Vec<_> = (0..lanes).collect();
        let mut rotations: Vec<_> = (0..lanes).collect();
        shuffle(&mut labels, rng);
        shuffle(&mut rotations, rng);
        for shift in rotations {
            if result.len() == count {
                break;
            }
            result.push((0..lanes).map(|position| labels[(position + shift) % lanes]).collect());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_prefix_balances_each_lane_position() {
        for lanes in 2..=8 {
            for seed in [0, 1, 6757, u64::MAX] {
                let mut rng = seed;
                let rows = rounds(lanes, 4 * lanes + 1, &mut rng);
                let mut counts = vec![vec![0; lanes]; lanes];
                for (round, row) in rows.iter().enumerate() {
                    let mut permutation = row.clone();
                    permutation.sort();
                    assert_eq!(permutation, (0..lanes).collect::<Vec<_>>());
                    for (position, &lane) in row.iter().enumerate() {
                        counts[lane][position] += 1;
                    }
                    let complete = (round + 1) / lanes;
                    for lane in &counts {
                        assert!(lane.iter().all(|&count| count == complete || count == complete + 1));
                        assert!(lane.iter().max().unwrap() - lane.iter().min().unwrap() <= 1);
                        if (round + 1) % lanes == 0 {
                            assert!(lane.iter().all(|&count| count == complete));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn reproducible_and_no_fixed_middle_lane() {
        let first = rounds(3, 8, &mut 6757);
        assert_eq!(first, rounds(3, 8, &mut 6757));
        assert_ne!(first, rounds(3, 8, &mut 1));
        let middle: Vec<_> = (0..3).map(|lane| first.iter().filter(|row| row[1] == lane).count()).collect();
        assert!(middle.iter().all(|count| (2..=3).contains(count)));
        assert!(rounds(3, 0, &mut 6757).is_empty());
    }
}
