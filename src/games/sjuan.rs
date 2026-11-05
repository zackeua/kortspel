use crate::cards::{Card, Rank, Suit, standard_deck};

pub struct Sjuan {
    hands: Vec<Vec<Card>>,
    board: [[bool; 13]; 4],
    current_player: usize,
    winner: Option<usize>,
    first_out: Option<usize>,
    rng: u64,
}

impl Sjuan {
    pub fn new(player_count: usize, seed: u64) -> Result<Self, &'static str> {
        if !(2..=5).contains(&player_count) {
            return Err("Sjuan kräver 2–5 spelare.");
        }

        let mut deck = standard_deck();
        let mut rng = if seed == 0 {
            0x9e37_79b9_7f4a_7c15
        } else {
            seed
        };
        for index in (1..deck.len()).rev() {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            deck.swap(index, (rng as usize) % (index + 1));
        }

        let mut hands = vec![Vec::new(); player_count];
        for (index, card) in deck.into_iter().enumerate() {
            hands[index % player_count].push(card);
        }
        for hand in &mut hands {
            hand.sort_by_key(|card| (suit_index(card.suit), rank_index(card.rank)));
        }

        Ok(Self {
            hands,
            board: [[false; 13]; 4],
            current_player: 0,
            winner: None,
            first_out: None,
            rng,
        })
    }

    pub fn current_player(&self) -> usize {
        self.current_player
    }

    pub fn active_player_count(&self) -> usize {
        self.hands.iter().filter(|hand| !hand.is_empty()).count()
    }

    pub fn hand(&self, player: usize) -> Option<&[Card]> {
        self.hands.get(player).map(Vec::as_slice)
    }

    pub fn has_card(&self, suit: Suit, rank: Rank) -> bool {
        self.board[suit_index(suit)][rank_index(rank)]
    }

    pub fn winner(&self) -> Option<usize> {
        self.winner
    }

    pub fn legal_indices(&self) -> Vec<usize> {
        if self.winner.is_some() {
            return Vec::new();
        }
        self.hands[self.current_player]
            .iter()
            .enumerate()
            .filter_map(|(index, card)| self.is_legal(*card).then_some(index))
            .collect()
    }

    pub fn play(&mut self, hand_index: usize) -> Result<(), &'static str> {
        if self.winner.is_some() {
            return Err("Spelet är redan slut.");
        }
        let Some(card) = self.hands[self.current_player].get(hand_index).copied() else {
            return Err("Det kortet finns inte på handen.");
        };
        if !self.is_legal(card) {
            return Err("Kortet kan inte läggas ännu.");
        }

        self.hands[self.current_player].remove(hand_index);
        self.board[suit_index(card.suit)][rank_index(card.rank)] = true;
        if self.hands[self.current_player].is_empty() {
            self.first_out.get_or_insert(self.current_player);
        }
        if self.active_player_count() <= 1 {
            self.winner = self.first_out;
        }
        if self.winner.is_none() {
            self.advance_turn();
        }
        Ok(())
    }

    pub fn pass(&mut self) -> Result<(), &'static str> {
        if self.winner.is_some() {
            return Err("Spelet är redan slut.");
        }
        if !self.legal_indices().is_empty() {
            return Err("Du måste lägga ett kort om du kan.");
        }
        let player = self.current_player;
        for opponent in 0..self.hands.len() {
            if opponent == player || self.hands[opponent].is_empty() {
                continue;
            }
            self.rng ^= self.rng << 13;
            self.rng ^= self.rng >> 7;
            self.rng ^= self.rng << 17;
            let card_index = self.rng as usize % self.hands[opponent].len();
            let card = self.hands[opponent].remove(card_index);
            self.hands[player].push(card);
            if self.hands[opponent].is_empty() {
                self.first_out.get_or_insert(opponent);
            }
        }
        self.hands[player].sort_by_key(|card| (suit_index(card.suit), rank_index(card.rank)));
        if self.active_player_count() <= 1 {
            self.winner = self.first_out;
        } else {
            self.advance_turn();
        }
        Ok(())
    }

    fn is_legal(&self, card: Card) -> bool {
        let rank = rank_index(card.rank);
        let suit = suit_index(card.suit);
        if rank == 6 {
            return !self.board[suit].iter().any(|placed| *placed);
        }
        (rank > 0 && self.board[suit][rank - 1]) || (rank < 12 && self.board[suit][rank + 1])
    }

    fn advance_turn(&mut self) {
        for _ in 0..self.hands.len() {
            self.current_player = (self.current_player + 1) % self.hands.len();
            if !self.hands[self.current_player].is_empty() {
                break;
            }
        }
    }
}

fn rank_index(rank: Rank) -> usize {
    match rank {
        Rank::Ace => 0,
        Rank::Two => 1,
        Rank::Three => 2,
        Rank::Four => 3,
        Rank::Five => 4,
        Rank::Six => 5,
        Rank::Seven => 6,
        Rank::Eight => 7,
        Rank::Nine => 8,
        Rank::Ten => 9,
        Rank::Jack => 10,
        Rank::Queen => 11,
        Rank::King => 12,
    }
}

fn suit_index(suit: Suit) -> usize {
    match suit {
        Suit::Clubs => 0,
        Suit::Diamonds => 1,
        Suit::Hearts => 2,
        Suit::Spades => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deals_all_cards_once_for_each_supported_player_count() {
        for count in 2..=5 {
            let game = Sjuan::new(count, 7).unwrap();
            let cards = game
                .hands
                .iter()
                .flatten()
                .copied()
                .collect::<std::collections::HashSet<_>>();
            assert_eq!(cards.len(), 52);
            assert_eq!(game.hands.iter().map(Vec::len).sum::<usize>(), 52);
        }
    }

    #[test]
    fn only_a_seven_starts_a_suit_then_adjacent_ranks_are_playable() {
        let mut game = Sjuan::new(2, 1).unwrap();
        let seven = Card {
            rank: Rank::Seven,
            suit: Suit::Clubs,
        };
        let six = Card {
            rank: Rank::Six,
            suit: Suit::Hearts,
        };
        let eight = Card {
            rank: Rank::Eight,
            suit: Suit::Clubs,
        };
        let five = Card {
            rank: Rank::Five,
            suit: Suit::Clubs,
        };
        game.hands[0] = vec![six, seven];
        game.hands[1] = vec![eight, five];

        assert_eq!(game.legal_indices(), vec![1]);
        game.play(1).unwrap();
        assert_eq!(game.current_player(), 1);
        assert_eq!(game.legal_indices(), vec![0]);
        game.play(0).unwrap();
        assert!(game.has_card(Suit::Clubs, Rank::Eight));
        assert!(game.legal_indices().is_empty());
        assert!(game.pass().is_ok());
    }

    #[test]
    fn six_can_extend_down_from_a_seven() {
        let mut game = Sjuan::new(2, 3).unwrap();
        let six = Card {
            rank: Rank::Six,
            suit: Suit::Clubs,
        };
        let seven = Card {
            rank: Rank::Seven,
            suit: Suit::Clubs,
        };
        game.hands[0] = vec![six, seven];
        game.hands[1] = vec![
            Card {
                rank: Rank::Eight,
                suit: Suit::Clubs,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Hearts,
            },
        ];

        assert_eq!(game.legal_indices(), vec![1]);
        game.play(1).unwrap();
        game.play(0).unwrap();
        assert_eq!(game.legal_indices(), vec![0]);
        game.play(0).unwrap();
        assert!(game.has_card(Suit::Clubs, Rank::Six));
    }

    #[test]
    fn cannot_pass_while_a_legal_card_is_available() {
        let mut game = Sjuan::new(2, 9).unwrap();
        let seven_index = game.hands[0]
            .iter()
            .position(|card| card.rank == Rank::Seven)
            .unwrap();
        game.play(seven_index).unwrap();
        assert_eq!(game.pass(), Err("Du måste lägga ett kort om du kan."));
    }

    #[test]
    fn passing_gives_one_card_from_each_active_opponent() {
        let mut game = Sjuan::new(3, 17).unwrap();
        game.hands[0] = vec![Card {
            rank: Rank::King,
            suit: Suit::Clubs,
        }];
        game.hands[1] = vec![
            Card {
                rank: Rank::Two,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Ace,
                suit: Suit::Clubs,
            },
        ];
        game.hands[2] = vec![
            Card {
                rank: Rank::Four,
                suit: Suit::Spades,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Diamonds,
            },
        ];

        game.pass().unwrap();

        assert_eq!(game.hands[0].len(), 3);
        assert_eq!(game.hands[1].len(), 1);
        assert_eq!(game.hands[2].len(), 1);
        assert_eq!(game.current_player(), 1);
        assert!(game.hands[0].windows(2).all(|pair| {
            (suit_index(pair[0].suit), rank_index(pair[0].rank))
                <= (suit_index(pair[1].suit), rank_index(pair[1].rank))
        }));
    }

    #[test]
    fn finished_players_are_skipped_until_only_one_remains() {
        let mut game = Sjuan::new(3, 21).unwrap();
        game.hands[0] = vec![Card {
            rank: Rank::Seven,
            suit: Suit::Clubs,
        }];
        game.hands[1] = vec![Card {
            rank: Rank::Seven,
            suit: Suit::Diamonds,
        }];
        game.hands[2] = vec![Card {
            rank: Rank::Seven,
            suit: Suit::Hearts,
        }];

        game.play(0).unwrap();
        assert_eq!(game.active_player_count(), 2);
        assert_eq!(game.winner(), None);
        assert_eq!(game.current_player(), 1);
        game.play(0).unwrap();
        assert_eq!(game.active_player_count(), 1);
        assert_eq!(game.winner(), Some(0));
    }
}
