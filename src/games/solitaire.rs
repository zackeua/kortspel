use crate::cards::{Card, Rank, Suit, standard_deck};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableauCard {
    pub card: Card,
    pub face_up: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    Waste,
    Tableau { column: usize, index: usize },
    Foundation(usize),
}

pub struct Solitaire {
    stock: Vec<Card>,
    waste: Vec<Card>,
    foundations: [Vec<Card>; 4],
    tableau: [Vec<TableauCard>; 7],
    selected: Option<Source>,
    status: String,
}

impl Solitaire {
    pub fn new(seed: u64) -> Self {
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

        let mut tableau: [Vec<TableauCard>; 7] = std::array::from_fn(|_| Vec::new());
        for (column, cards) in tableau.iter_mut().enumerate() {
            for index in 0..=column {
                cards.push(TableauCard {
                    card: deck.pop().expect("a standard deck has enough cards"),
                    face_up: index == column,
                });
            }
        }

        Self {
            stock: deck,
            waste: Vec::new(),
            foundations: std::array::from_fn(|_| Vec::new()),
            tableau,
            selected: None,
            status: "Klicka på talongen för att dra tre kort.".to_owned(),
        }
    }

    pub fn stock_count(&self) -> usize {
        self.stock.len()
    }

    pub fn waste_visible(&self) -> &[Card] {
        let start = self.waste.len().saturating_sub(3);
        &self.waste[start..]
    }

    pub fn tableau(&self) -> &[Vec<TableauCard>; 7] {
        &self.tableau
    }

    pub fn foundation_top(&self, index: usize) -> Option<Card> {
        self.foundations.get(index)?.last().copied()
    }

    pub fn is_waste_selected(&self) -> bool {
        self.selected == Some(Source::Waste)
    }

    pub fn is_tableau_selected(&self, column: usize, index: usize) -> bool {
        matches!(
            self.selected,
            Some(Source::Tableau {
                column: selected_column,
                index: selected_index
            }) if selected_column == column && index >= selected_index
        )
    }

    pub fn is_foundation_selected(&self, index: usize) -> bool {
        self.selected == Some(Source::Foundation(index))
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn has_won(&self) -> bool {
        self.foundations.iter().all(|pile| pile.len() == 13)
    }

    pub fn no_moves_before_stock_recycle(&self) -> bool {
        if self.stock_count() > 0 || self.has_won() {
            return false;
        }

        if self.waste.last().is_some_and(|card| {
            self.can_move_to_foundation(*card) || self.can_move_to_tableau(*card, None)
        }) {
            return false;
        }

        for (column, cards) in self.tableau.iter().enumerate() {
            let Some(first_face_up) = cards.iter().position(|card| card.face_up) else {
                continue;
            };
            for index in first_face_up..cards.len() {
                let card = cards[index].card;
                if index + 1 == cards.len() && self.can_move_to_foundation(card) {
                    return false;
                }
                if self.can_move_to_tableau(card, Some(column)) {
                    return false;
                }
            }
        }

        self.foundations.iter().all(|foundation| {
            foundation
                .last()
                .is_none_or(|card| !self.can_move_to_tableau(*card, None))
        })
    }

    pub fn click_stock(&mut self) {
        self.selected = None;
        if !self.stock.is_empty() {
            for _ in 0..3 {
                if let Some(card) = self.stock.pop() {
                    self.waste.push(card);
                }
            }
            self.status = "Upp till tre kort har dragits till slänghögen.".to_owned();
        } else if !self.waste.is_empty() {
            self.stock.extend(self.waste.drain(..).rev());
            self.status = "Slänghögen har blandats tillbaka.".to_owned();
        } else {
            self.status = "Det finns inga fler kort att dra.".to_owned();
        }
    }

    pub fn click_waste(&mut self) {
        if self.waste.is_empty() {
            self.status = "Slänghögen är tom.".to_owned();
            return;
        }
        if self.selected == Some(Source::Waste) {
            self.selected = None;
            self.status = "Markeringen har tagits bort.".to_owned();
        } else {
            self.selected = Some(Source::Waste);
            self.status = "Välj en grund eller en kolumn att flytta kortet till.".to_owned();
        }
    }

    pub fn click_tableau(&mut self, column: usize, index: usize) {
        let Some(card) = self
            .tableau
            .get(column)
            .and_then(|cards| cards.get(index))
            .copied()
        else {
            self.status = "Den kolumnen finns inte.".to_owned();
            return;
        };
        if !card.face_up {
            self.status = "Välj ett öppet kort.".to_owned();
            return;
        }

        let source = Source::Tableau { column, index };
        if self.selected == Some(source) {
            self.selected = None;
            self.status = "Markeringen har tagits bort.".to_owned();
            return;
        }
        if self.selected.is_none() {
            self.selected = Some(source);
            self.status = "Välj en kolumn att flytta kortet till.".to_owned();
            return;
        }

        if self.move_to_tableau(column) {
            return;
        }

        if matches!(self.selected, Some(Source::Tableau { column: from, .. }) if from == column) {
            self.selected = Some(source);
        }
    }

    pub fn click_empty_tableau(&mut self, column: usize) {
        if column >= self.tableau.len() {
            self.status = "Den kolumnen finns inte.".to_owned();
        } else if self.tableau[column].is_empty() {
            self.move_to_tableau(column);
        }
    }

    pub fn click_foundation(&mut self, foundation: usize) {
        if foundation >= self.foundations.len() {
            self.status = "Den grunden finns inte.".to_owned();
            return;
        }
        let target_source = Source::Foundation(foundation);
        let Some(source) = self.selected else {
            if self.foundations[foundation].is_empty() {
                self.status = "Välj ett ess eller ett öppet kort först.".to_owned();
            } else {
                self.selected = Some(target_source);
                self.status = "Välj en kolumn att flytta kortet till.".to_owned();
            }
            return;
        };
        if source == target_source {
            self.selected = None;
            self.status = "Markeringen har tagits bort.".to_owned();
            return;
        }
        let Some(card) = self.source_cards(source).last().copied() else {
            self.selected = None;
            return;
        };
        let target = &self.foundations[foundation];
        let can_place = match target.last() {
            None => card.rank == Rank::Ace && suit_index(card.suit) == foundation,
            Some(top) => {
                suit_index(top.suit) == suit_index(card.suit)
                    && rank_value(card.rank) == rank_value(top.rank) + 1
            }
        };
        if !can_place || self.source_cards(source).len() != 1 {
            self.status = "Bara nästa kort i samma färg kan läggas på grunden.".to_owned();
            return;
        }
        let moved = self
            .take_source(source)
            .expect("selected source contains a card");
        self.foundations[foundation].push(moved[0]);
        self.selected = None;
        self.status = "Kortet flyttades till grunden.".to_owned();
    }

    fn move_to_tableau(&mut self, column: usize) -> bool {
        let Some(source) = self.selected else {
            return false;
        };
        if matches!(source, Source::Tableau { column: from, .. } if from == column) {
            return false;
        }
        let moving = self.source_cards(source);
        let Some(first) = moving.first() else {
            self.status = "Välj ett öppet kort först.".to_owned();
            return false;
        };
        if !self.can_place_on_tableau(*first, column) {
            self.status = "Lägg korten i fallande ordning med växlande färg.".to_owned();
            return false;
        }

        let moved = self
            .take_source(source)
            .expect("selected source contains cards");
        self.tableau[column].extend(moved.into_iter().map(|card| TableauCard {
            card,
            face_up: true,
        }));
        self.selected = None;
        self.status = "Kortet flyttades.".to_owned();
        true
    }

    fn can_move_to_foundation(&self, card: Card) -> bool {
        let foundation = &self.foundations[suit_index(card.suit)];
        match foundation.last() {
            None => card.rank == Rank::Ace,
            Some(top) => rank_value(card.rank) == rank_value(top.rank) + 1,
        }
    }

    fn can_move_to_tableau(&self, card: Card, source_column: Option<usize>) -> bool {
        self.tableau.iter().enumerate().any(|(column, _)| {
            if source_column == Some(column) {
                return false;
            }
            self.can_place_on_tableau(card, column)
        })
    }

    fn can_place_on_tableau(&self, card: Card, column: usize) -> bool {
        match self.tableau[column]
            .last()
            .map(|tableau_card| tableau_card.card)
        {
            None => card.rank == Rank::King,
            Some(top) => {
                is_red(card.suit) != is_red(top.suit)
                    && rank_value(card.rank) + 1 == rank_value(top.rank)
            }
        }
    }

    fn source_cards(&self, source: Source) -> Vec<Card> {
        match source {
            Source::Waste => self.waste.last().copied().into_iter().collect(),
            Source::Tableau { column, index } => self
                .tableau
                .get(column)
                .and_then(|cards| cards.get(index..))
                .map(|cards| cards.iter().map(|card| card.card).collect())
                .unwrap_or_default(),
            Source::Foundation(index) => self.foundations[index]
                .last()
                .copied()
                .into_iter()
                .collect(),
        }
    }

    fn take_source(&mut self, source: Source) -> Option<Vec<Card>> {
        let cards = match source {
            Source::Waste => vec![self.waste.pop()?],
            Source::Tableau { column, index } => {
                let cards = self.tableau.get_mut(column)?;
                let moved = cards.split_off(index);
                if let Some(top) = cards.last_mut() {
                    top.face_up = true;
                }
                moved.into_iter().map(|card| card.card).collect()
            }
            Source::Foundation(index) => vec![self.foundations.get_mut(index)?.pop()?],
        };
        Some(cards)
    }
}

fn rank_value(rank: Rank) -> u8 {
    match rank {
        Rank::Ace => 1,
        Rank::Two => 2,
        Rank::Three => 3,
        Rank::Four => 4,
        Rank::Five => 5,
        Rank::Six => 6,
        Rank::Seven => 7,
        Rank::Eight => 8,
        Rank::Nine => 9,
        Rank::Ten => 10,
        Rank::Jack => 11,
        Rank::Queen => 12,
        Rank::King => 13,
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

fn is_red(suit: Suit) -> bool {
    matches!(suit, Suit::Diamonds | Suit::Hearts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deal_has_seven_columns_and_twenty_four_stock_cards() {
        let game = Solitaire::new(1);
        assert_eq!(game.stock_count(), 24);
        assert_eq!(
            game.tableau
                .iter()
                .map(|column| column.len())
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 6, 7]
        );
        assert!(game.tableau.iter().enumerate().all(|(column, cards)| {
            cards
                .iter()
                .enumerate()
                .all(|(index, card)| card.face_up == (index == column))
        }));
    }

    #[test]
    fn drawing_then_recycling_stock_restores_all_cards() {
        let mut game = Solitaire::new(7);
        for _ in 0..8 {
            game.click_stock();
        }
        assert_eq!(game.stock_count(), 0);
        assert_eq!(game.waste.len(), 24);
        game.click_stock();
        assert_eq!(game.stock_count(), 24);
        assert!(game.waste.is_empty());
    }

    #[test]
    fn stock_draws_three_cards_and_only_shows_the_top_three_waste_cards() {
        let mut game = Solitaire::new(8);
        let stock_before = game.stock.clone();

        game.click_stock();
        assert_eq!(game.stock_count(), 21);
        assert_eq!(game.waste.len(), 3);
        assert_eq!(
            game.waste_visible(),
            &[stock_before[23], stock_before[22], stock_before[21]]
        );
        assert_eq!(game.waste_visible().last(), Some(&stock_before[21]));

        game.click_stock();
        assert_eq!(game.stock_count(), 18);
        assert_eq!(game.waste.len(), 6);
        assert_eq!(game.waste_visible().len(), 3);
    }

    #[test]
    fn final_stock_draw_contains_only_cards_left() {
        let mut game = Solitaire::new(9);
        for _ in 0..7 {
            game.click_stock();
        }
        assert_eq!(game.stock_count(), 3);
        game.click_stock();
        assert_eq!(game.stock_count(), 0);
        assert_eq!(game.waste.len(), 24);
    }

    #[test]
    fn tableau_moves_require_alternating_color_and_reveal_the_previous_card() {
        let mut game = Solitaire::new(10);
        game.tableau[0] = vec![
            TableauCard {
                card: Card {
                    rank: Rank::King,
                    suit: Suit::Clubs,
                },
                face_up: false,
            },
            TableauCard {
                card: Card {
                    rank: Rank::Seven,
                    suit: Suit::Hearts,
                },
                face_up: true,
            },
        ];
        game.tableau[1] = vec![TableauCard {
            card: Card {
                rank: Rank::Eight,
                suit: Suit::Spades,
            },
            face_up: true,
        }];
        game.tableau[2].clear();

        game.click_tableau(0, 1);
        game.click_tableau(1, 0);
        assert_eq!(game.tableau[1].len(), 2);
        assert!(game.tableau[0][0].face_up);

        game.click_tableau(0, 0);
        game.click_empty_tableau(2);
        assert_eq!(game.tableau[2][0].card.rank, Rank::King);
        assert!(game.tableau[0].is_empty());
    }

    #[test]
    fn aces_move_to_foundations_and_foundation_cards_can_return_to_tableau() {
        let mut game = Solitaire::new(12);
        game.tableau[0] = vec![TableauCard {
            card: Card {
                rank: Rank::Ace,
                suit: Suit::Clubs,
            },
            face_up: true,
        }];
        game.click_tableau(0, 0);
        game.click_foundation(0);
        assert_eq!(game.foundation_top(0).unwrap().rank, Rank::Ace);
        assert!(game.tableau[0].is_empty());

        game.tableau[1] = vec![TableauCard {
            card: Card {
                rank: Rank::Two,
                suit: Suit::Hearts,
            },
            face_up: true,
        }];
        game.click_foundation(0);
        game.click_tableau(1, 0);
        assert!(game.foundations[0].is_empty());
        assert_eq!(game.tableau[1].len(), 2);
        assert_eq!(game.tableau[1][1].card.rank, Rank::Ace);
    }

    #[test]
    fn no_moves_are_reported_only_after_stock_is_empty() {
        let mut game = Solitaire::new(13);
        game.stock.clear();
        game.waste.clear();
        for (index, column) in game.tableau.iter_mut().enumerate() {
            *column = vec![TableauCard {
                card: Card {
                    rank: Rank::King,
                    suit: [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades][index % 4],
                },
                face_up: true,
            }];
        }

        assert!(game.no_moves_before_stock_recycle());

        game.stock.push(Card {
            rank: Rank::Ace,
            suit: Suit::Clubs,
        });
        assert!(!game.no_moves_before_stock_recycle());

        game.stock.clear();
        game.tableau[0].clear();
        assert!(!game.no_moves_before_stock_recycle());
    }
}
