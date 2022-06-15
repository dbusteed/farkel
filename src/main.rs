use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::color;
use rand::{Rng, thread_rng};
use pad::{PadStr, Alignment};
use std::io::stdout;
use std::collections::HashMap;
use std::io;
use std::process::exit;
use std::{thread, time::Duration};

const ASCII_ONE: u8 = 49;
const ASCII_SIX: u8 = 54;
const DEFAULT_COLOR: color::Fg<color::Reset> = color::Fg(color::Reset);
const WINNING_SCORE: usize = 5000;
const N_BOTS: usize = 2;

// the expected value and chance of farkel (no score)
// when rolling N dice (where the N-1 is the index
// of this list). Each entry is as follows: 
//   [ Expected Score, P(Farkel), P(~Farkel) ]
const ROLL_STATS: [(f64, f64, f64); 6] = [
    (24., 2./3., 1./3.),
    (49., 4./9., 5./9.),
    (83., 5./18., 13./18.),
    (132., 20./127., 107./127.),
    (203., 1./13., 12./13.),
    (384., 5./216., 211./216.),
];  

const BOT_NAMES: [&str; 11] = [
    "Billy", "Charlie", "Debbie", 
    "Eddie", "Frank", "Jerry", "George",
    "Timmy", "Kathy", "Martha", "Suzy"
];

struct Player {
    name: String,
    total: usize,
    tmp: usize,
    bot: bool,
}

impl Player {
    fn new(name: &str, bot: bool) -> Self {
        Self {
            name: String::from(name),
            total: 0,
            tmp: 0,
            bot,
        }
    }
}

fn roll_die() -> u8 {
    thread_rng().gen_range(1..=6)
}

fn roll_n_die(n: u8) -> Vec<u8> {
    (0..n)
        .map(|_| roll_die())
        .collect()
}

fn draw_dice(dice: &Vec<u8>) {
    // simple "amimation" of rolling the dice
    for _ in 0..5 {
        for i in 0..dice.len() {
            draw_die(&roll_die(), i as u8, DEFAULT_COLOR);
        }
        thread::sleep(Duration::from_millis(75));
    }

    // draw the actual rolled dice
    for (i, d) in dice.iter().enumerate() {
        draw_die(d, i as u8, DEFAULT_COLOR);
    }
}

fn draw_blank_dice(n: &u8) {
    for i in 0..*n {
        draw_die(&0, i, DEFAULT_COLOR);
    }
}

fn draw_die<C>(val: &u8, pos: u8, color: color::Fg<C>)
    where C: color::Color,
{
    let y = 2;
    let x = ((pos*6) + 2) as u16;
    
    let middle = if val == &0 {
        "│   │".to_string()
    } else {
        format!("│ {} │", val)
    };
    
    draw("╭───╮", x, y, &color);
    draw(&middle, x, y+1, &color);
    draw("╰───╯", x, y+2, &color);
}

fn hold_dice(mut held: HashMap<u8, bool>, pos: u8, val: &u8) -> HashMap<u8, bool> {
    held.insert(pos, !held[&pos]);
    
    match held.get(&pos) {
        Some(b) => {
            if *b {
                draw_die(val, pos, color::Fg(color::LightGreen));
            } else {
                draw_die(val, pos, DEFAULT_COLOR);
            }
        },
        None => {},
    }
    
    held
}

fn draw_player_scores(players: &Vec<Player>, turn: &usize) {
    let y_offest = 2;

    for (i, p) in players.iter().enumerate() {
        let y = (i + y_offest) as u16;
        let s = p.total.to_string().pad(7, ' ', Alignment::Right, true);
        if i == *turn {
            draw(&s, 42, y, &color::Fg(color::LightGreen));
            draw(&p.name, 50, y, &color::Fg(color::LightGreen));
        } else {
            draw(&s, 42, y, &DEFAULT_COLOR);
            draw(&p.name, 50, y, &DEFAULT_COLOR);
        }
    }

    let y2 = (y_offest + players.len() + 1) as u16;
    let s = players[*turn].tmp.to_string().pad(7, ' ', Alignment::Right, true);
    draw(&s, 42, y2, &DEFAULT_COLOR);
    draw("Round Score", 50, y2, &DEFAULT_COLOR);
}

fn draw<C>(content: &str, x: u16, y: u16, color: &color::Fg<C>)
    where C: color::Color,
{
    println!(
        "{}{}{}{}{}",
        termion::cursor::Goto(x, y),
        color,
        content,
        color::Fg(color::Reset),
        termion::style::Reset,
    );
}

fn clear() {
    println!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
}

fn clear_messages() {
    println!("{}{}", termion::cursor::Goto(1, 6), termion::clear::CurrentLine);
    println!("{}{}", termion::cursor::Goto(1, 7), termion::clear::CurrentLine);
    println!("{}{}", termion::cursor::Goto(1, 8), termion::clear::CurrentLine);
    println!("{}{}", termion::cursor::Goto(1, 9), termion::clear::CurrentLine);
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
            tot_score = 0;
            break
        } else if rice.is_empty() {
            break
        }

        size1 = rice.len();
    }
    
    tot_score
}

fn bot_score_dice(dice: &mut Vec<u8>) -> (usize, usize) {
    let mut size1 = dice.len();
    let mut tot_score = 0;
    let mut tmp_score;
    let mut rice = (*dice).clone();
    loop {
        (tmp_score, rice) = find_points(&mut rice);
        tot_score += tmp_score;
        if size1 == rice.len() {
            break
        } else if rice.is_empty() {
            break
        }

        size1 = rice.len();
    }
    
    (tot_score, dice.len() - rice.len())
}

fn draw_intro_title() {
    let title: &str = r#"  _______   ___      ______   __  ___  _______  __      
 |   ____| /   \    |   _  \ |  |/  / |   ____||  |     
 |  |__   /  ^  \   |  |_)  ||  '  /  |  |__   |  |     
 |   __| /  /_\  \  |      / |    /   |   __|  |  |     
 |  |   /  _____  \ |  |\  \ |  .  \  |  |____ |  `---.
 |__|  /__/     \__\| _| \._\|__|\__\ |_______||______|
"#;
    for (i, line) in title.split('\n').enumerate() {
        let y = (i + 1) as u16;
        println!("{}", termion::style::Bold);
        draw(line, 1, y, &color::Fg(color::LightGreen))
    }
    
    println!("{}", termion::style::Reset);
}

fn pass_the_dice(players: &Vec<Player>, turn: &usize, n_dice: &u8) -> bool {
    // can't pass if you don't have 500 points
    if players[*turn].tmp + players[*turn].total < 500 {
        return false;
    }

    let idx = (*n_dice - 1) as usize;
    let tmp = players[*turn].tmp as f64;
    let roll_ev = 
        ROLL_STATS[idx].0 +
        (tmp * -ROLL_STATS[idx].1) +
        (tmp * ROLL_STATS[idx].2);
    
    roll_ev <= tmp
}

fn bot_round(
    players: &mut Vec<Player>, 
    turn: &mut usize,
    n_dice: &mut u8,
    score: &mut usize,
    sleep_base: u64,
) -> bool {
    *n_dice = if *n_dice == 0 { 6 } else { *n_dice };
    draw_blank_dice(n_dice);
    
    draw(&format!("{} deciding to roll or pass...", players[*turn].name), 2, 6, &DEFAULT_COLOR);

    thread::sleep(Duration::from_secs(1 * sleep_base));

    let pass_dice = pass_the_dice(players, turn, n_dice);
    if pass_dice {
        players[*turn].total += players[*turn].tmp;
        players[*turn].tmp = 0; 
        *turn = (*turn + 1) % players.len();
        return true
    }

    let mut dice = roll_n_die(*n_dice);

    clear();
    draw_player_scores(&players, &turn);
    draw_dice(&dice);

    let (possible_points, _) = find_points(&mut dice);
    if possible_points == 0 {
        draw("Farkel!", 2, 6, &DEFAULT_COLOR);
        players[*turn].tmp = 0; 
        *turn = (*turn + 1) % players.len();
        thread::sleep(Duration::from_secs(1 * sleep_base));
        return true
    }

    thread::sleep(Duration::from_secs(1 * sleep_base));
        
    let n_used_dice;
    (*score, n_used_dice) = bot_score_dice(&mut dice);
    
    players[*turn].tmp += *score;
    draw(&format!("{} scored {} points!", players[*turn].name, score), 2, 6, &DEFAULT_COLOR);

    *n_dice -= n_used_dice as u8;
    return false
}

fn player_round(
    players: &mut Vec<Player>, 
    turn: &mut usize,
    n_dice: &mut u8,
    score: &mut usize,
) -> bool {
    *n_dice = if *n_dice == 0 { 6 } else { *n_dice };
    
    draw_blank_dice(n_dice);
    
    draw("[Enter] to roll, [Backspace] to pass", 2, 6, &DEFAULT_COLOR);
    if *score != 0 {
        draw(&format!("- You scored {} points", score), 2, 8, &DEFAULT_COLOR);
        draw(&format!("- {} dice remaining", n_dice), 2, 9, &DEFAULT_COLOR);
    }

    let mut pass_dice = false;
    loop {
        if let Some(key) = io::stdin().lock().keys().next() {
            match key.unwrap() {
                Key::Char('\n') => break,
                Key::Backspace => {
                    if players[*turn].tmp + players[*turn].total < 500 {
                        draw("- You need at least 500 points to get on the", 2, 8, &DEFAULT_COLOR);
                        draw("   scoreboard it doesn't make sense to pass", 2, 9, &DEFAULT_COLOR);
                    } else {
                        pass_dice = true;
                        break
                    }
                },
                Key::Char('q') => quit(),
                Key::Esc => quit(),
                _ => {}
            }
        }
    }

    if pass_dice {
        players[*turn].total += players[*turn].tmp;
        players[*turn].tmp = 0; 
        *turn = (*turn + 1) % players.len();
        return true
    }

    // roll the dice
    let mut dice = roll_n_die(*n_dice);
    let mut tmp_dice;
    
    // setup the map to keep track of held die
    let mut held: HashMap<u8, bool> = HashMap::new();
    for i in 0..dice.len() {
        held.insert(i as u8, false);
    }

    clear();
    draw_player_scores(&players, &turn);
    draw_dice(&dice);

    let (possible_points, _) = find_points(&mut dice);
    if possible_points == 0 {
        draw("Farkel!", 2, 6, &color::Fg(color::LightRed));
        players[*turn].tmp = 0; 
        *turn = (*turn + 1) % players.len();
        thread::sleep(Duration::from_secs(1));
        return true
    }

    draw("Select dice to score with 1-6 keys", 2, 6, &DEFAULT_COLOR);

    loop {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                let kkey = key.unwrap();
                held = match kkey {
                    Key::Char('q') => {
                        println!("{}", termion::cursor::Show);
                        exit(1);
                    },
                    Key::Char('\n') => break,
                    Key::Char(c) => {
                        match c as u8 {
                            ASCII_ONE..=ASCII_SIX => {
                                let pos = (c as u8) - ASCII_ONE;
                                if pos < dice.len() as u8 {
                                    hold_dice(held, pos, &dice[pos as usize])
                                } else {
                                    held
                                }
                            }
                            _ => held,
                        }
                    }
                    _ => held
                }
            }
        }

        tmp_dice = held.iter()
            .filter(|&(_, v)| *v)
            .map(|(&k, _)| k)
            .map(|k| dice[k as usize])
            .collect::<Vec<u8>>();
        
        
        *score = score_dice(&mut tmp_dice);
        match *score {
            0 => draw("- Not a valid combination", 2, 8, &DEFAULT_COLOR),
            s => {
                players[*turn].tmp += s;
                draw_player_scores(&players, &turn);
                break
            }
        }
    }

    *n_dice -= tmp_dice.len() as u8;

    return false
}

fn quit() {
    println!("{}", termion::cursor::Show);
    println!("{}", termion::clear::All);
    println!("{}", termion::cursor::Goto(1, 1));
    exit(0);
}

fn main() {
    let mut _stdout = stdout().into_raw_mode().unwrap();
    println!("{}", termion::cursor::Hide);
    clear();
    
    if N_BOTS > BOT_NAMES.len() {
        panic!("N_BOTS > BOT_NAMES");
    }

    let mut game = true;
    draw_intro_title();

    let menu_options = vec![
        "Classic",
        "Quick Mode",
        "Quit",
    ];

    let n_options = menu_options.len();
    let mut selection = 0;
    let mut arrow: &str;

    // game menu
    loop {
        for (i, opt) in menu_options.iter().enumerate() {
            arrow = "  ";
            if i == selection {
                println!("{}", termion::style::Bold);
                arrow = "> ";
            }
            draw(&format!("{}{}", arrow, opt), 10, 8 + i as u16, &color::Fg(color::Reset));
        }

        // make a selection
        if let Some(key) = io::stdin().lock().keys().next() {
            match key.unwrap() {
                Key::Up => {
                    if selection == 0 {
                        selection = n_options - 1;
                    } else {
                        selection = (selection - 1) % n_options;
                    }
                },
                Key::Down => {
                    selection = (selection + 1) % n_options;
                },
                Key::Char('\n') => break,
                _ => {}
            }
        }
    }

    // game settings
    let mut sleep_base: u64 = 1;
    
    match selection {
        1 => {
            sleep_base = 0;
        },
        2 => quit(),
        _ => {}
    }
    
    let mut turn = 0;

    let mut players: Vec<Player> = vec![];
    players.push(Player::new("Davis", false)); // TODO use system username
    
    // make bots
    for _ in 0..N_BOTS {
        let idx = thread_rng().gen_range(1..BOT_NAMES.len());
        players.push(Player::new(BOT_NAMES[idx], true));
    }

    // loop until someone wins the game
    while game {
        let mut n_dice: u8 = 6;
        let mut score = 0;
        
        // loop for each round
        loop {
            clear();
            draw_player_scores(&players, &turn);

            if players[turn].bot {
                match bot_round(&mut players, &mut turn, &mut n_dice, &mut score, sleep_base) {
                    true => break,
                    _ => {}
                }
            } else {
                match player_round(&mut players, &mut turn, &mut n_dice, &mut score) {
                    true => break,
                    _ => {}
                }
            }
        }

        for p in players.iter() {
            if p.total >= WINNING_SCORE {
                game = false;
            }
        }
        
        // TODO everyone gets one more turn
    }

    // find the winner
    let winner = players.iter().max_by_key(|p| p.total).unwrap();

    draw_player_scores(&players, &turn);
    clear_messages();
    draw(&format!("{} won!", winner.name), 2, 6, &DEFAULT_COLOR);
    draw("(press any key to exit)", 2, 8, &DEFAULT_COLOR);
    io::stdin().lock().keys().next();

    // reset the terminal
    println!("{}", termion::cursor::Show);
    println!("{}", termion::clear::All);
    println!("{}", termion::cursor::Goto(1, 1));
}
