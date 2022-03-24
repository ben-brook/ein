use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::VecDeque;
use std::fmt;
use std::io;

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
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

trait ColoredCard {
    fn get_color(&self) -> Color;
}

#[derive(Debug)]
struct NumberCard {
    color: Color,
    number: u8,
}

impl ColoredCard for NumberCard {
    fn get_color(&self) -> Color {
        self.color
    }
}

#[derive(Debug)]
struct ActionCard {
    color: Color,
    action: Action,
}

impl ColoredCard for ActionCard {
    fn get_color(&self) -> Color {
        self.color
    }
}

#[derive(Debug)]
struct WildCard {
    action: WildAction,
}

#[derive(Debug)]
enum Card {
    Number(NumberCard),
    Action(ActionCard),
    Wild(WildCard),
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

fn transfer_cards(global_deck: &mut VecDeque<Card>, deck: &mut Vec<Card>, amount: u8) -> bool {
    for _ in 0..amount {
        match global_deck.pop_front() {
            Some(card) => {
                deck.push(card);
            }
            None => return false,
        }
    }
    true
}

fn gen_global_deck() -> VecDeque<Card> {
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
            global_deck.push(Card::Wild(WildCard { action }));
        }
    }

    let mut rng = thread_rng();
    global_deck.shuffle(&mut rng);

    VecDeque::from(global_deck)
}

fn init_players(bot_count: u8, global_deck: &mut VecDeque<Card>) -> Vec<Player> {
    let mut players = Vec::new();

    for i in 0..=bot_count {
        let mut deck = Vec::new();
        transfer_cards(global_deck, &mut deck, 7);
        players.push(Player {
            deck,
            is_human: i == 0,
        });
    }

    players
}

fn main() {
    let mut global_deck = gen_global_deck();

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

    let players = init_players(bot_count, &mut global_deck);
    let human = &mut players[0];

    let mut deck_display = String::new();
    for card in human.deck {
        deck_display.push_str(card);
    }
    println!("Your deck is {}");
}
