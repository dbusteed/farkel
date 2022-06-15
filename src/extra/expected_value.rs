use rand::{Rng, thread_rng};
use std::collections::HashMap;

fn roll_die() -> u8 {
    thread_rng().gen_range(1..=6)
}

fn roll_n_die(n: u8) -> Vec<u8> {
    (0..n)
        .map(|_| roll_die())
        .collect()
}

fn find_points(dice: &mut Vec<u8>) -> (usize, Vec<u8>) {
    let mut counts: HashMap<u8, u8> = HashMap::new();
    let straight: Vec<u8> = vec![1, 2, 3, 4, 5, 6];

    for n in dice.iter() {
        *counts.entry(*n).or_insert(0) += 1;
    }
    
    let count_values: Vec<u8> = counts.clone().into_values().collect();
    
    let mut tmp = (*dice).clone();
    let mut sorted_dice = (*dice).clone();
    sorted_dice.sort();

    if sorted_dice == straight {
        (1500, vec![])
    
    } else if count_values == vec![6] {
        (3000, vec![])
    
    } else if count_values == vec![3, 3] {
        (2500, vec![])
        
    } else if count_values == vec![2, 2, 2] {
        (1500, vec![])
        
    } else if count_values.contains(&5) {
        let val = counts.into_iter()
            .filter(|(_, v)| *v == 5)
            .map(|(k, _)| k)
            .collect::<Vec<u8>>();
        let remain = tmp.into_iter()
            .filter(|&n| n != val[0])
            .collect::<Vec<u8>>();

        (2000, remain)

    } else if count_values.contains(&4) {
        let val = counts.into_iter()
            .filter(|(_, v)| *v == 4)
            .map(|(k, _)| k)
            .collect::<Vec<u8>>();
        let remain = tmp.into_iter()
            .filter(|&n| n != val[0])
            .collect::<Vec<u8>>();
        
        (1000, remain)

    } else if count_values.contains(&3) {
        let val = counts.into_iter()
            .filter(|(_, v)| *v == 3)
            .map(|(k, _)| k)
            .collect::<Vec<u8>>();
        let remain = tmp.into_iter()
            .filter(|&n| n != val[0])
            .collect::<Vec<u8>>();

        if val[0] == 1 {
            (300, remain)    
        } else {
            (val[0] as usize * 100, remain)
        }

    } else if tmp.contains(&1) {
        tmp.remove(tmp.iter().position(|x| *x == 1).unwrap());
        (100, tmp)

    } else if tmp.contains(&5) {
        tmp.remove(tmp.iter().position(|x| *x == 5).unwrap());
        (50, tmp)

    } else {
        (0, tmp)
    }
}

fn score_dice(dice: &mut Vec<u8>) -> usize {
    let mut size1 = dice.len();
    let mut tot_score = 0;
    let mut tmp_score;
    let mut rice = (*dice).clone();
    loop {
        (tmp_score, rice) = find_points(&mut rice);
        tot_score += tmp_score;
        if size1 == rice.len() {
            break;
        } else if rice.is_empty() {
            break;
        }

        size1 = rice.len();
    }
    
    tot_score
}

fn average(numbers: Vec<usize>) -> usize {
    let sum: usize = numbers.iter().sum();
    let count = numbers.len() as usize;
    sum / count
}

fn main() {
    for n in 1..=6 {
        let mut scores = vec![];
        for _ in 0..10_000_000 {
            scores.push(score_dice(&mut roll_n_die(n)));
        }
        println!("{:?} {:?}", n, average(scores));
    }
}

//
// Expected Values
//
// 1 25
// 2 50
// 3 83
// 4 132
// 5 203
// 6 384

//
// Farkel chance (from wikipedia)
//
// 1	1 in 1.5 (2 in 3)
// 2	1 in 2.25 (4 in 9)
// 3	1 in 3.6 (5 in 18)
// 4	1 in 6.35 (20 in 127)
// 5	1 in 13
// 6	1 in 43.2 (5 in 216)