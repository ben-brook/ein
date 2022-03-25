use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::VecDeque;
use std::fmt;
use std::io;
use std::thread::Thread;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
    Yellow,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Color::Red => "Red",
                Color::Green => "Green",
                Color::Blue => "Blue",
                Color::Yellow => "Yellow",
            }
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Action {
    Draw2,
    Reverse,
    Skip,
}

#[derive(Clone, Copy, Debug)]
enum WildAction {
    ChangeColor,
    Draw4,
}

#[derive(Debug, Clone, Copy)]
struct NumberCard {
    color: Color,
    number: u8,
}

#[derive(Debug, Clone, Copy)]
struct ActionCard {
    color: Color,
    action: Action,
}

#[derive(Debug, Clone, Copy)]
struct WildCard {
    color: Option<Color>,
    action: WildAction,
}

#[derive(Debug, Clone, Copy)]
enum Card {
    Number(NumberCard),
    Action(ActionCard),
    Wild(WildCard),
}

impl Card {
    fn accepts(&self, other: &Card) -> bool {
        match self {
            Card::Number(NumberCard { color, number }) => match other {
                Card::Number(NumberCard {
                    color: other_color,
                    number: other_number,
                }) => other_color == color || other_number == number,
                Card::Action(ActionCard {
                    color: other_color,
                    action: _,
                }) => other_color == color,
                Card::Wild(_) => true,
            },

            Card::Action(ActionCard { color, action }) => match other {
                Card::Number(NumberCard {
                    color: other_color,
                    number: _,
                }) => other_color == color,
                Card::Action(ActionCard {
                    color: other_color,
                    action: other_action,
                }) => other_color == color || other_action == action,
                Card::Wild(_) => true,
            },

            Card::Wild(WildCard { color, action: _ }) => match other {
                Card::Number(NumberCard {
                    color: other_color,
                    number: _,
                })
                | Card::Action(ActionCard {
                    color: other_color,
                    action: _,
                }) => other_color == &color.unwrap(),
                Card::Wild(_) => true,
            },
        }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Card::Wild(card) => write!(
                f,
                "{}",
                match card.action {
                    WildAction::ChangeColor => "Change Color",
                    WildAction::Draw4 => "Draw 4",
                }
            ),
            Card::Number(card) => {
                write!(f, "{} {}", card.color, card.number)
            }
            Card::Action(card) => {
                write!(
                    f,
                    "{} {}",
                    card.color,
                    match card.action {
                        Action::Draw2 => "Draw 2",
                        Action::Reverse => "Reverse",
                        Action::Skip => "Skip",
                    }
                )
            }
        }
    }
}

#[derive(Debug)]
struct Player {
    deck: Vec<Card>,
    is_human: bool,
}

const COLORS: [Color; 4] = [Color::Red, Color::Green, Color::Blue, Color::Yellow];
const ACTIONS: [Action; 3] = [Action::Draw2, Action::Reverse, Action::Skip];
const WILD_ACTIONS: [WildAction; 2] = [WildAction::ChangeColor, WildAction::Draw4];
const MAX_BOTS: u8 = 9;
const NUM_CARDS: usize = 108;

fn transfer_cards(
    global_deck: &mut VecDeque<Card>,
    deck: &mut Vec<Card>,
    amount: u8,
    disallows_wild: bool,
    rng: &mut ThreadRng,
) -> bool {
    let mut has_transferred = true;
    loop {
        for _ in 0..amount {
            match global_deck.pop_front() {
                Some(card) => {
                    if !disallows_wild {
                        deck.push(card);
                        continue;
                    }
                    if let Card::Wild(_) = card {
                        let mut vec = Vec::from(global_deck.clone());
                        vec.push(card);
                        vec.shuffle(rng);
                        *global_deck = VecDeque::from(vec);
                        has_transferred = false;
                        break;
                    }
                }
                None => return false,
            }
        }
        if has_transferred {
            break;
        }
        has_transferred = true;
    }

    true
}

fn gen_global_deck(rng: &mut ThreadRng) -> VecDeque<Card> {
    let mut global_deck = Vec::with_capacity(NUM_CARDS);

    for color in COLORS {
        // We add one 0 card.
        global_deck.push(Card::Number(NumberCard { color, number: 0 }));

        // We add the rest of the cards twice.
        for _ in 0..2 {
            for number in 1..=9 {
                global_deck.push(Card::Number(NumberCard { color, number }));
            }

            for action in ACTIONS {
                global_deck.push(Card::Action(ActionCard { color, action }));
            }
        }
    }

    for action in WILD_ACTIONS {
        for _ in 0..4 {
            global_deck.push(Card::Wild(WildCard {
                action,
                color: None,
            }));
        }
    }

    global_deck.shuffle(rng);

    VecDeque::from(global_deck)
}

fn init_players(
    bot_count: u8,
    global_deck: &mut VecDeque<Card>,
    rng: &mut ThreadRng,
) -> Vec<Player> {
    let mut players = Vec::new();

    for i in 0..=bot_count {
        let mut deck = Vec::new();
        transfer_cards(global_deck, &mut deck, 7, false, rng);
        players.push(Player {
            deck,
            is_human: i == 0,
        });
    }

    players
}

fn get_deck_display(deck: &[Card]) -> String {
    let mut deck_display = String::new();

    for (i, card) in deck.iter().enumerate() {
        deck_display.push_str(&format!(
            "[{}] {card}{}",
            i + 1,
            if i == deck.len() - 2 {
                ", and "
            } else if i == deck.len() - 1 {
                "."
            } else {
                ", "
            }
        ));
    }

    deck_display
        .trim_end_matches(|x| x == ' ' || x == ',')
        .to_string()
}

fn main() {
    let mut rng = thread_rng();
    let mut global_deck = gen_global_deck(&mut rng);
    let mut discarded = Vec::new();
    transfer_cards(&mut global_deck, &mut discarded, 1, true, &mut rng);

    println!("Enter bot count:");
    let mut buf = String::new();
    let bot_count = loop {
        io::stdin().read_line(&mut buf).unwrap();
        if let Ok(count) = buf.trim().parse::<u8>() {
            if (1..=MAX_BOTS).contains(&count) {
                break count;
            }
            println!("Bot count must be between 1 and {MAX_BOTS} inclusively. Try again:");
        } else {
            println!("You must input a standalone integer. Try again:");
        }

        buf.clear();
    };

    let mut dir = 1;
    let mut is_hot = true;
    let mut players = init_players(bot_count, &mut global_deck, &mut rng);

    let mut cur_idx = 0;
    loop {
        let player = &mut players[cur_idx];
        let top_discarded = &discarded[discarded.len() - 1];
        let is_special = match top_discarded {
            Card::Action(ActionCard { color: _, action }) => match action {
                Action::Draw2 => {
                    transfer_cards(&mut global_deck, &mut player.deck, 2, false, &mut rng);
                    println!(
                        "You picked up two cards. Your new deck is:\n{}",
                        get_deck_display(&player.deck)
                    );
                    if player.is_human {}
                    true
                }
                Action::Skip => true,
                Action::Reverse => false,
            },
            Card::Wild(WildCard { color: _, action }) => match action {
                WildAction::Draw4 => {
                    transfer_cards(&mut global_deck, &mut player.deck, 4, false, &mut rng);
                    true
                }
                WildAction::ChangeColor => false,
            },
            Card::Number(_) => false,
        };
        if !(is_hot && is_special) {
            let can_play = player
                .deck
                .iter()
                .filter(|x| top_discarded.accepts(x))
                .count()
                == 0;

            let card_idx;
            if player.is_human {
                println!(
                    "Your turn! Your deck contains {}\nWhich card do you play?",
                    get_deck_display(&player.deck)
                );

                card_idx = loop {
                    buf.clear();
                    io::stdin().read_line(&mut buf).unwrap();
                    if let Ok(card_num) = buf.trim().parse::<usize>() {
                        if !(1..=player.deck.len()).contains(&card_num) {
                            println!("Card not listed.");
                            continue;
                        }
                        let card_idx = card_num - 1;
                        let card = &player.deck[card_idx];
                        if top_discarded.accepts(card) {
                            break card_idx;
                        }
                        println!("Card cannot be placed on a {top_discarded}.");
                    } else {
                        println!("You must input a standalone integer. Try again:");
                    }
                };
            } else {
            }
        }

        cur_idx += dir;
        if !is_hot {
            is_hot = true;
        }
    }
}
