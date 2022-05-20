#![warn(clippy::pedantic)]

use core::time;
use std::thread;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use rand::{prelude::ThreadRng, seq::SliceRandom};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Color {
    Red,
    Blue,
    Green,
    Yellow,
}
const COLORS: [Color; 4] = [Color::Red, Color::Blue, Color::Green, Color::Yellow];

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Action {
    Draw2,
    Reverse,
    Skip,
}
const ACTIONS: [Action; 3] = [Action::Draw2, Action::Reverse, Action::Skip];

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum WildAction {
    ChangeColor,
    Draw4,
}
const WILD_ACTIONS: [WildAction; 2] = [WildAction::ChangeColor, WildAction::Draw4];

enum PlayResult {
    Win,
    Place(Option<Color>),
    NoPlace,
    Starvation,
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Card {
    Number { number: u8, color: Color },
    Action { action: Action, color: Color },
    Wild(WildAction),
}

impl Card {
    fn accepts(&self, other: &Card, wild_color: Option<Color>) -> bool {
        match [self, other] {
            [Card::Number { color, number }, Card::Number {
                color: other_color,
                number: other_number,
            }] => color == other_color || number == other_number,

            [Card::Number { color, .. }, Card::Action {
                color: other_color, ..
            }]
            | [Card::Action { color, .. }, Card::Number {
                color: other_color, ..
            }] => color == other_color,

            [Card::Action { color, action }, Card::Action {
                action: other_action,
                color: other_color,
            }] => color == other_color || action == other_action,

            [_, Card::Wild(_)] => true,

            [Card::Wild(_), Card::Number { number: _, color } | Card::Action { action: _, color }] => {
                *color == wild_color.unwrap()
            }
        }
    }
}

const MAX_BOTS: u8 = 9;
const INITIAL_CARDS_PER_PLAYER: u8 = 7;

fn gen_draw_pile(rng: &mut ThreadRng) -> Vec<Card> {
    let mut draw_pile = Vec::with_capacity(112);

    for color in COLORS {
        draw_pile.push(Card::Number { number: 0, color });

        for _ in 0..2 {
            for number in 1..=9 {
                draw_pile.push(Card::Number { number, color });
            }

            for action in ACTIONS {
                draw_pile.push(Card::Action { action, color });
            }
        }
    }
    for wild_action in WILD_ACTIONS {
        for _ in 0..4 {
            draw_pile.push(Card::Wild(wild_action));
        }
    }

    draw_pile.shuffle(rng);
    draw_pile
}

fn transfer_cards(
    draw_pile: &mut Vec<Card>,
    discard_pile: &mut Vec<Card>,
    deck: &mut Vec<Card>,
    amount: u8,
    rng: &mut ThreadRng,
) -> bool {
    for _ in 0..amount {
        loop {
            match draw_pile.pop() {
                Some(card) => {
                    deck.push(card);
                    break;
                }
                None => {
                    // Discard pile size should never be 0 since there'll always
                    // be a top card.
                    if discard_pile.len() == 1 {
                        // There are no more cards left to play with.
                        return true;
                    }
                    for card in discard_pile.drain(..discard_pile.len() - 1) {
                        draw_pile.push(card);
                    }
                    draw_pile.shuffle(rng);
                }
            }
        }
    }

    false
}

fn init_discard_pile(discard_pile: &mut Vec<Card>, draw_pile: &mut Vec<Card>, rng: &mut ThreadRng) {
    while matches!(draw_pile.last().unwrap(), Card::Wild(_)) {
        draw_pile.shuffle(rng);
    }
    discard_pile.push(draw_pile.pop().unwrap());
}

fn init_players(
    bot_count: u8,
    draw_pile: &mut Vec<Card>,
    discard_pile: &mut Vec<Card>,
    rng: &mut ThreadRng,
) -> Vec<Box<dyn Player>> {
    let mut players: Vec<Box<dyn Player>> = Vec::new();

    for i in 0..=bot_count {
        let mut deck = Vec::new();
        transfer_cards(
            draw_pile,
            discard_pile,
            &mut deck,
            INITIAL_CARDS_PER_PLAYER,
            rng,
        );
        if i == 0 {
            // players.push(Box::new(Human { deck }));
        } else {
            players.push(Box::new(Bot { deck }));
        }
    }

    players
}

fn ask_bot_count(buf: &mut String) -> u8 {
    println!("Enter bot count:");
    loop {
        std::io::stdin().read_line(buf).unwrap();
        if let Ok(count) = buf.trim().parse::<u8>() {
            if (1..=MAX_BOTS).contains(&count) {
                break count;
            }
            println!("Bot count must be between 1 and {MAX_BOTS} inclusively. Try again:");
        } else {
            println!("You must input a standalone integer. Try again:");
        }

        buf.clear();
    }
}

// https://stackoverflow.com/a/48491021
impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        match rng.gen_range(0..=3) {
            0 => Color::Red,
            1 => Color::Blue,
            2 => Color::Yellow,
            _ => Color::Green, // 3
        }
    }
}

fn start() {
    let mut rng = rand::thread_rng();
    let mut draw_pile = gen_draw_pile(&mut rng);
    let mut discard_pile = Vec::new();
    let mut dir = 1; // Inverted by Reverse cards
    let mut is_hot = true; // Was the top card from the last player?
    let mut wild_color = None;
    let mut cur_idx = 0;

    let mut buf = String::new();
    let mut players = init_players(
        ask_bot_count(&mut buf),
        &mut draw_pile,
        &mut discard_pile,
        &mut rng,
    );
    init_discard_pile(&mut discard_pile, &mut draw_pile, &mut rng);
    let player_count = i8::try_from(players.len()).unwrap();

    let play_result = loop {
        let result = players[cur_idx].play(
            &mut draw_pile,
            &mut discard_pile,
            &mut dir,
            is_hot,
            wild_color,
            cur_idx,
            &mut rng,
        );
        match result {
            PlayResult::Place(new_wild_color) => {
                is_hot = true;
                wild_color = new_wild_color;
            }
            PlayResult::NoPlace => {
                is_hot = false;
            }
            _ => break result,
        }

        cur_idx = (cur_idx + usize::try_from(player_count + dir).unwrap()) % players.len();
    };

    if matches!(play_result, PlayResult::Win) {
        println!(
            "Game over: {}!",
            if cur_idx == 0 {
                String::from("you win")
            } else {
                format!("Bot {cur_idx} wins")
            }
        );
    } else {
        println!("Game over: ran out of cards to play with");
    }
}

fn main() {
    start();
}

trait Player {
    fn play(
        &mut self,
        draw_pile: &mut Vec<Card>,
        discard_pile: &mut Vec<Card>,
        dir: &mut i8,
        is_hot: bool,
        wild_color: Option<Color>,
        player_idx: usize,
        rng: &mut ThreadRng,
    ) -> PlayResult;
}

struct Human {
    deck: Vec<Card>,
}
impl Player for Human {
    fn play(
        &mut self,
        draw_pile: &mut Vec<Card>,
        discard_pile: &mut Vec<Card>,
        dir: &mut i8,
        is_hot: bool,
        wild_color: Option<Color>,
        player_idx: usize,
        rng: &mut ThreadRng,
    ) -> PlayResult {
        PlayResult::NoPlace
    }
}

struct Bot {
    deck: Vec<Card>,
}
impl Player for Bot {
    fn play(
        &mut self,
        draw_pile: &mut Vec<Card>,
        discard_pile: &mut Vec<Card>,
        dir: &mut i8,
        is_hot: bool,
        wild_color: Option<Color>,
        player_idx: usize,
        rng: &mut ThreadRng,
    ) -> PlayResult {
        if is_hot {
            // Deal with consequential cards from the last move.
            match discard_pile[discard_pile.len() - 1] {
                Card::Action { action, .. } => match action {
                    Action::Skip => return PlayResult::NoPlace,
                    Action::Draw2 => {
                        if transfer_cards(draw_pile, discard_pile, &mut self.deck, 2, rng) {
                            return PlayResult::Starvation;
                        };
                        announce_bot_move(format!("Bot {player_idx} draws two cards."));
                        return PlayResult::NoPlace;
                    }
                    _ => {}
                },

                Card::Wild(wild_action) => {
                    if matches!(wild_action, WildAction::Draw4) {
                        if transfer_cards(draw_pile, discard_pile, &mut self.deck, 4, rng) {
                            return PlayResult::Starvation;
                        };
                        announce_bot_move(format!("Bot {player_idx} draws four cards."));
                        return PlayResult::NoPlace;
                    }
                }

                _ => {}
            }
        }

        // Find a card to play.
        let mut possible_idxs = Vec::new();
        for (idx, card) in self.deck.iter().enumerate() {
            if discard_pile[discard_pile.len() - 1].accepts(card, wild_color) {
                possible_idxs.push(idx);
            }
        }
        let mut chosen_idx = possible_idxs.choose(rng).copied();

        if chosen_idx == None {
            // Pick a card from the pile.
            if transfer_cards(draw_pile, discard_pile, &mut self.deck, 1, rng) {
                return PlayResult::Starvation;
            }
            announce_bot_move(format!("Bot {player_idx} draws a card."));
            if discard_pile[discard_pile.len() - 1].accepts(self.deck.last().unwrap(), wild_color) {
                // We can play it.
                chosen_idx = Some(self.deck.len() - 1);
            }
        }

        if let Some(idx) = chosen_idx {
            discard_pile.push(self.deck.swap_remove(idx));

            let mut new_wild_color = None;
            let played_card = discard_pile.last().unwrap();
            announce_bot_move(format!("Bot {player_idx} plays a {played_card:?}."));
            match played_card {
                Card::Action { action, .. } => {
                    if matches!(action, Action::Reverse) {
                        *dir = -*dir;
                    }
                }
                Card::Wild(_) => {
                    let new_wild_color_contents = rng.gen();
                    new_wild_color = Some(new_wild_color_contents);
                    announce_bot_move(format!(
                        "Bot {player_idx} chooses {new_wild_color_contents:?} as the new colour."
                    ));
                }
                _ => {}
            }

            if self.deck.len() == 0 {
                PlayResult::Win
            } else {
                PlayResult::Place(new_wild_color)
            }
        } else {
            PlayResult::NoPlace
        }
    }
}

fn announce_bot_move(announcement: String) {
    println!("{announcement}");
    thread::sleep(time::Duration::from_millis(500));
}
