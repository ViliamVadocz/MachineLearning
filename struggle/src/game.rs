use crate::messages::{GameMessage, LastAction};
use crate::card::*;

pub struct Game {
    pub my_hand: Vec<Card>,
    pub deck_size: u32,
    pub center: Vec<Card>,
    pub unseen_cards: Vec<Card>,
    pub players: Vec<PlayerInfo>,
    pub current_player_index: usize,
    pub has_moves: bool
}

impl Game {
    pub fn new() -> Game {
        

        Game {
            my_hand: Vec::new(),
            deck_size: 56,
            center: Vec::new(),
            unseen_cards: Vec::new(),
            players: Vec::new(),
            current_player_index: 0,
            has_moves: true,
        }
    }

    pub fn start(&mut self, players: Vec<String>) {
        self.players = players.iter().enumerate().map(
            |(i, name)| PlayerInfo {
                name: name.to_string(),
                index: i,
                hand: vec![CardPlace::Unknown, CardPlace::Unknown]
            }).collect();
    }

    pub fn update(&mut self, message: GameMessage) {
        self.my_hand = message.hand.iter().map(|card| Card::from(card.to_string())).collect();
        // parse game state
        let game_state = message.game_state;
        self.has_moves = game_state.has_moves;
        self.deck_size = game_state.deck_size;
        self.current_player_index = game_state.current_player;
        // let current_player = &game_state.players[self.current_player_index];
        let previous_player_index = (self.current_player_index + self.players.len() - 1) % self.players.len();
        let previous_player = &game_state.players[previous_player_index];

        // use previous action to update knowledge
        if let Some(previous_action) = &previous_player.last_action {
            println!("{0} played: {1:?}", previous_player.name, previous_action);
            let action_taken = Action::from(previous_action);
        } else {
            println!("{} played: no previous action", previous_player.name);
        }

        println!("---");

        // print centre and deck
        let center = &game_state.center;
        println!("center: {:?}", center);
        let deck_size = game_state.deck_size;
        println!("deck size: {}", deck_size);
    }

    fn get_start_deck() -> Vec<Card> {
        let mut start_deck: Vec<Card> = Vec::new();
        for &suit in [Suit::Club, Suit::Heart, Suit::Spade, Suit::Diamond].iter() {
            start_deck.extend((2..15).map(|value| Card::SuitCard {suit, value}).collect::<Vec<Card>>());
        }
        start_deck.extend((1..5).map(|id| Card::Joker {id}));
        start_deck
    }
}

pub struct PlayerInfo {
    name: String,
    index: usize,
    hand: Vec<CardPlace>
}

pub enum Action {
    Draw(Option<Card>),
    Trick(Vec<Card>)
}

impl Action {
    fn from(action: &LastAction) -> Action {
        match &action.action_type.as_str() {
            &"draw" => {
                match &action.card {
                    Some(card) => Action::Draw(Some(Card::from(card.to_string()))),
                    None => Action::Draw(None)
                }
            },
            &"play" => Action::Trick(
                action.cards.iter().map(|card| Card::from(card.to_string())).collect()
            ),
            _ => panic!("invalid move type in last move")
        }
    }
}