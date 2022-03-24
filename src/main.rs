use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io;

#[derive(Clone, Copy, Debug)]
enum Color {
    Red,
    Green,
    Blue,
    Yellow,
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

#[derive(Debug)]
struct Player {
    deck: Vec<Card>,
    is_human: bool,
}

const COLORS: [Color; 4] = [Color::Red, Color::Green, Color::Blue, Color::Yellow];
const ACTIONS: [Action; 3] = [Action::Draw2, Action::Reverse, Action::Skip];
const WILD_ACTIONS: [WildAction; 2] = [WildAction::ChangeColor, WildAction::Draw4];
const MAX_BOTS: u8 = 9;

fn transfer_cards(global_deck: &mut Vec<Card>, deck: &mut Vec<Card>, amount: u8) -> bool {
    for _ in 0..amount {
        match global_deck.pop() {
            Some(card) => {
                deck.push(card);
            }
            None => return false,
        }
    }
    true
}

fn main() {
    let mut global_deck: Vec<Card> = Vec::new();

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

    println!("Enter bot count:");
    let mut buf = String::new();
    let bot_count = loop {
        io::stdin().read_line(&mut buf).unwrap();
        if let Ok(count) = buf.trim().parse::<u8>() {
            if count <= MAX_BOTS && count >= 1 {
                break count;
            } else {
                println!("Bot count must be between 1 and {MAX_BOTS} inclusively. Try again:");
            }
        } else {
            println!("You must input a standalone integer. Try again:");
        }
        buf.clear();
    };

    let mut players = Vec::new();
    for i in 0..=bot_count {
        let mut deck = Vec::new();
        transfer_cards(&mut global_deck, &mut deck, 7);
        players.push(Player {
            deck,
            is_human: i == 0,
        });
    }

    for (i, player) in players.iter().enumerate() {
        println!(
            "The cards of player {}, {}, are {:?}",
            i,
            if player.is_human {
                "the human"
            } else {
                "a bot"
            },
            player.deck
        );
        println!("");
    }
}
