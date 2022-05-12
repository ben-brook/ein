#![warn(clippy::pedantic)]

use rand::{prelude::ThreadRng, seq::SliceRandom};
use std::collections::HashSet;

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
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Card {
    Number { number: u8, color: Color },
    Action { action: Action, color: Color },
    Wild(WildAction),
}

impl Card {
    fn accepts(&self, other: &Card, wild_color: Option<Color>) -> bool {
        match self {
            Card::Number { color, number } => match other {
                Card::Number {
                    color: other_color,
                    number: other_number,
                } => other_color == color || other_number == number,
                Card::Action {
                    color: other_color,
                    action: _,
                } => other_color == color,
                Card::Wild(_) => true,
            },

            Card::Action { color, action } => match other {
                Card::Number {
                    color: other_color,
                    number: _,
                } => other_color == color,
                Card::Action {
                    color: other_color,
                    action: other_action,
                } => other_color == color || other_action == action,
                Card::Wild(_) => true,
            },

            Card::Wild(action) => match other {
                Card::Number {
                    color: other_color,
                    number: _,
                }
                | Card::Action {
                    color: other_color,
                    action: _,
                } => *other_color == wild_color.unwrap(),
                Card::Wild(_) => true,
            },
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
    deck: &mut HashSet<Card>,
    amount: u8,
    rng: &mut ThreadRng,
) -> bool {
    for _ in 0..amount {
        loop {
            match draw_pile.pop() {
                Some(card) => {
                    deck.insert(card);
                    break;
                }
                None => {
                    if discard_pile.is_empty() {
                        // There are no more cards left to play with.
                        return false;
                    }
                    discard_pile.shuffle(rng);
                    draw_pile.swap_with_slice(discard_pile);
                }
            }
        }
    }

    true
}

fn init_discard_pile(
    discard_pile: &mut Vec<Card>,
    draw_pile: &mut Vec<Card>,
    rng: &mut ThreadRng,
) -> Vec<Card> {
    loop {
        if matches!(draw_pile.last().unwrap(), Card::Wild(_)) {
            draw_pile.shuffle(rng);
        } else {
            discard_pile.push(draw_pile.pop().unwrap());
        }
    }
}

fn init_players(
    bot_count: u8,
    draw_pile: &mut Vec<Card>,
    discard_pile: &mut Vec<Card>,
    rng: &mut ThreadRng,
) -> Vec<Box<dyn Player>> {
    let mut players: Vec<Box<dyn Player>> = Vec::new();

    for i in 0..=bot_count {
        let mut deck = HashSet::new();
        transfer_cards(
            draw_pile,
            discard_pile,
            &mut deck,
            INITIAL_CARDS_PER_PLAYER,
            rng,
        );
        if i == 0 {
            players.push(Box::new(Human { deck }));
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

fn main() {
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
    let mut discard_pile = init_discard_pile(&mut discard_pile, &mut draw_pile, &mut rng);

    loop {
        match players[cur_idx].play(
            &mut draw_pile,
            &mut discard_pile,
            &mut dir,
            is_hot,
            wild_color,
        ) {
            PlayResult::Win => break,
            PlayResult::Place(new_wild_color) => {
                is_hot = true;
                wild_color = new_wild_color;
            }
            PlayResult::NoPlace => {
                is_hot = false;
            }
        }

        cur_idx = (cur_idx + usize::try_from(i8::try_from(players.len()).unwrap() + dir).unwrap())
            % players.len();
    }
}

trait Player {
    fn play(
        &mut self,
        draw_pile: &mut Vec<Card>,
        discard_pile: &mut Vec<Card>,
        dir: &mut i8,
        is_hot: bool,
        wild_color: Option<Color>,
    ) -> PlayResult;
}

struct Human {
    deck: HashSet<Card>,
}
impl Player for Human {
    fn play(
        &mut self,
        draw_pile: &mut Vec<Card>,
        discard_pile: &mut Vec<Card>,
        dir: &mut i8,
        is_hot: bool,
        wild_color: Option<Color>,
    ) -> PlayResult {
        PlayResult::NoPlace
    }
}

struct Bot {
    deck: HashSet<Card>,
}
impl Player for Bot {
    fn play(
        &mut self,
        draw_pile: &mut Vec<Card>,
        discard_pile: &mut Vec<Card>,
        dir: &mut i8,
        is_hot: bool,
        wild_color: Option<Color>,
    ) -> PlayResult {
        let top = discard_pile.last().unwrap();
        let mut chosen_idx = None;

        for (idx, card) in self.deck.iter().enumerate() {
            if top.accepts(card, wild_color) {
                chosen_idx = Some(idx);
            }
        }

        if let Some(idx) = chosen_idx {
            place(idx, self, discard_pile, dir);
            PlayResult::Place(None)
        } else {
            PlayResult::NoPlace
        }
    }
}

fn place(card_idx: usize, player: &mut impl Player, discard_pile: &mut [Card], dir: &mut i8) {}
