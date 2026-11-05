//! Reusable Macroquad rendering for standard playing cards.

use crate::{Card, Rank, Suit};
use macroquad::prelude::*;

pub const CARD_WIDTH: f32 = 100.0;
pub const CARD_HEIGHT: f32 = 142.0;

const INK: Color = Color::new(0.08, 0.1, 0.13, 1.0);

/// Visual emphasis applied when drawing a face-up card.
#[derive(Clone, Copy, Default)]
pub struct CardStyle {
    pub selected: bool,
    pub focused: bool,
}

/// Draws a face-up card inside `rect`.
pub fn draw_face(rect: Rect, card: Card, style: CardStyle) {
    draw_face_with_theme(rect, card, style, 0);
}

pub fn draw_face_with_theme(rect: Rect, card: Card, style: CardStyle, theme: usize) {
    let theme = theme.min(5);
    let scale = (rect.h / CARD_HEIGHT).min(rect.w / CARD_WIDTH);
    let suit = suit_color(card.suit);
    let (paper_color, theme_accent) = match theme {
        1 => (
            Color::new(0.83, 0.94, 0.89, 1.0),
            Color::new(0.12, 0.43, 0.33, 1.0),
        ),
        2 => (
            Color::new(1.0, 0.87, 0.81, 1.0),
            Color::new(0.76, 0.31, 0.25, 1.0),
        ),
        3 => (
            Color::new(0.88, 0.93, 1.0, 1.0),
            Color::new(0.1, 0.2, 0.48, 1.0),
        ),
        4 => (
            Color::new(0.96, 0.91, 0.76, 1.0),
            Color::new(0.44, 0.29, 0.14, 1.0),
        ),
        5 => (
            Color::new(0.97, 0.9, 0.97, 1.0),
            Color::new(0.48, 0.2, 0.49, 1.0),
        ),
        _ => (
            Color::new(0.98, 0.96, 0.87, 1.0),
            Color::new(0.12, 0.16, 0.2, 1.0),
        ),
    };
    let paper = if style.selected {
        Color::new(1.0, 0.95, 0.72, 1.0)
    } else {
        paper_color
    };
    let edge = if style.selected {
        GREEN
    } else if style.focused {
        GOLD
    } else {
        theme_accent
    };

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, edge);
    draw_rectangle(
        rect.x + 3.0 * scale,
        rect.y + 3.0 * scale,
        rect.w - 6.0 * scale,
        rect.h - 6.0 * scale,
        paper,
    );
    draw_rectangle(
        rect.x + 7.0 * scale,
        rect.y + 7.0 * scale,
        rect.w - 14.0 * scale,
        4.0 * scale,
        suit,
    );
    draw_rectangle(
        rect.x + 7.0 * scale,
        rect.y + rect.h - 11.0 * scale,
        rect.w - 14.0 * scale,
        4.0 * scale,
        suit,
    );
    match theme {
        1 => draw_rectangle_lines(
            rect.x + 5.0 * scale,
            rect.y + 5.0 * scale,
            rect.w - 10.0 * scale,
            rect.h - 10.0 * scale,
            scale,
            theme_accent,
        ),
        2 => draw_circle(
            rect.x + rect.w - 11.0 * scale,
            rect.y + rect.h - 20.0 * scale,
            3.0 * scale,
            theme_accent,
        ),
        3 => {
            for offset in [13.0, 25.0, 37.0] {
                draw_line(
                    rect.x + offset * scale,
                    rect.y + 5.0 * scale,
                    rect.x + offset * scale,
                    rect.y + 8.0 * scale,
                    scale,
                    theme_accent,
                );
            }
        }
        4 => draw_rectangle_lines(
            rect.x + 5.0 * scale,
            rect.y + 5.0 * scale,
            rect.w - 10.0 * scale,
            rect.h - 10.0 * scale,
            2.0 * scale,
            theme_accent,
        ),
        5 => {
            draw_circle(
                rect.x + rect.w - 11.0 * scale,
                rect.y + rect.h - 20.0 * scale,
                4.0 * scale,
                theme_accent,
            );
            draw_circle(
                rect.x + rect.w - 11.0 * scale,
                rect.y + rect.h - 20.0 * scale,
                2.0 * scale,
                paper,
            );
        }
        _ => {}
    }
    let corner_pixel = if theme == 0 { 3.0 * scale } else { scale };
    draw_rank_label(
        rank_label(card.rank),
        rect.x + 10.0 * scale,
        rect.y + (if theme == 0 { 14.0 } else { 10.0 }) * scale,
        corner_pixel,
        theme,
        INK,
        paper,
    );
    draw_pixel_pattern(
        rect.x + rect.w - 21.0 * scale,
        rect.y + 10.0 * scale,
        2.0 * scale,
        suit_pattern(card.suit),
        suit,
    );

    if has_face_portrait(card.rank) {
        let portrait_scale = scale * 1.15;
        draw_card_mascot(
            rect.x + rect.w / 2.0 - 17.5 * portrait_scale,
            rect.y + 37.0 * scale,
            card.rank,
            portrait_scale,
            theme,
        );
    } else if card.rank == Rank::Ace {
        draw_card_mascot(
            rect.x + rect.w / 2.0 - 17.5 * scale,
            rect.y + 55.0 * scale,
            card.rank,
            scale,
            theme,
        );
    } else {
        let label = rank_label(card.rank);
        let columns = rank_glyph_width(label, theme);
        let rank_pixel = ((rect.w - 20.0 * scale) / columns).clamp(2.0 * scale, 10.0 * scale);
        let rank_width = rank_text_width(label, rank_pixel, theme);
        draw_rank_label(
            label,
            rect.x + (rect.w - rank_width) / 2.0,
            rect.y + 53.0 * scale,
            rank_pixel,
            theme,
            INK,
            paper,
        );
    }

    draw_pixel_pattern(
        rect.x + rect.w / 2.0 - 10.5 * scale,
        rect.y + 119.0 * scale,
        1.5 * scale,
        suit_pattern(card.suit),
        suit,
    );
}

fn rank_glyph_width(text: &str, theme: usize) -> f32 {
    let columns = if theme == 0 { 3.0 } else { 5.0 };
    let spacing = 1.0;
    text.chars().count() as f32 * (columns + spacing) - spacing
}

fn rank_text_width(text: &str, pixel: f32, theme: usize) -> f32 {
    if theme == 0 {
        pixel_text_width(text, pixel)
    } else {
        rank_glyph_width(text, theme) * pixel
    }
}

fn draw_rank_label(
    text: &str,
    x: f32,
    y: f32,
    pixel: f32,
    theme: usize,
    color: Color,
    paper: Color,
) {
    if theme == 0 {
        draw_pixel_text(text, x, y + pixel, pixel, color);
        return;
    }

    let mut cursor_x = x;
    for character in text.chars() {
        let glyph = themed_glyph(character, theme);
        for (row, bits) in glyph.iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    if theme == 2 || theme == 5 {
                        draw_circle(
                            cursor_x + (column as f32 + 0.5) * pixel,
                            y + (row as f32 + 0.5) * pixel,
                            pixel * 0.48,
                            color,
                        );
                        if theme == 5 {
                            draw_circle(
                                cursor_x + (column as f32 + 0.5) * pixel,
                                y + (row as f32 + 0.5) * pixel,
                                pixel * 0.2,
                                paper,
                            );
                        }
                    } else if theme == 4 {
                        draw_rectangle(
                            cursor_x + column as f32 * pixel,
                            y + row as f32 * pixel,
                            pixel * 1.25,
                            pixel * 1.15,
                            color,
                        );
                    } else {
                        draw_rectangle(
                            cursor_x + column as f32 * pixel,
                            y + row as f32 * pixel,
                            pixel,
                            pixel,
                            color,
                        );
                    }
                }
            }
        }
        cursor_x += pixel * 6.0;
    }
}

fn themed_glyph(character: char, theme: usize) -> [u8; 7] {
    let serif = match character {
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        'A' => [
            0b11111, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        _ => [0; 7],
    };
    match theme {
        1 => serif,
        4 => std::array::from_fn(|row| match row {
            0 | 3 | 6 => serif[row] | 0b10001,
            1 | 5 => serif[row] | 0b01010,
            _ => serif[row],
        }),
        3 => seven_segment_glyph(character),
        _ => match character {
            '0' => [
                0b01110, 0b11111, 0b11011, 0b11011, 0b11011, 0b11111, 0b01110,
            ],
            '1' => [
                0b00100, 0b01100, 0b11100, 0b00100, 0b00100, 0b00100, 0b11111,
            ],
            '2' => [
                0b01110, 0b11111, 0b00011, 0b00110, 0b01100, 0b11000, 0b11111,
            ],
            '3' => [
                0b11110, 0b00011, 0b00011, 0b01110, 0b00011, 0b00011, 0b11110,
            ],
            '4' => [
                0b00011, 0b00111, 0b01111, 0b11011, 0b11111, 0b00011, 0b00011,
            ],
            '5' => [
                0b11111, 0b11000, 0b11000, 0b11110, 0b00011, 0b00011, 0b11110,
            ],
            '6' => [
                0b01110, 0b11000, 0b11000, 0b11110, 0b11011, 0b11011, 0b01110,
            ],
            '7' => [
                0b11111, 0b00011, 0b00110, 0b01100, 0b11000, 0b11000, 0b11000,
            ],
            '8' => [
                0b01110, 0b11011, 0b11011, 0b01110, 0b11011, 0b11011, 0b01110,
            ],
            '9' => [
                0b01110, 0b11011, 0b11011, 0b01111, 0b00011, 0b00011, 0b01110,
            ],
            'A' => [
                0b01110, 0b11111, 0b11011, 0b11011, 0b11111, 0b11011, 0b11011,
            ],
            'J' => [
                0b00111, 0b00011, 0b00011, 0b00011, 0b11011, 0b11011, 0b01110,
            ],
            'Q' => [
                0b01110, 0b11011, 0b11011, 0b11011, 0b11111, 0b01110, 0b00011,
            ],
            'K' => [
                0b11011, 0b11011, 0b11110, 0b11100, 0b11110, 0b11011, 0b11011,
            ],
            _ => [0; 7],
        },
    }
}

fn seven_segment_glyph(character: char) -> [u8; 7] {
    match character {
        '0' => [
            0b11111, 0b11011, 0b10101, 0b10101, 0b10101, 0b11011, 0b11111,
        ],
        '1' => [
            0b00110, 0b01110, 0b00110, 0b00110, 0b00110, 0b00110, 0b11111,
        ],
        '2' => [
            0b11111, 0b00011, 0b00011, 0b11111, 0b11000, 0b11000, 0b11111,
        ],
        '3' => [
            0b11111, 0b00011, 0b00011, 0b01111, 0b00011, 0b00011, 0b11111,
        ],
        '4' => [
            0b11011, 0b11011, 0b11011, 0b11111, 0b00011, 0b00011, 0b00011,
        ],
        '5' => [
            0b11111, 0b11000, 0b11000, 0b11111, 0b00011, 0b00011, 0b11111,
        ],
        '6' => [
            0b11111, 0b11000, 0b11000, 0b11111, 0b11011, 0b11011, 0b11111,
        ],
        '7' => [
            0b11111, 0b00011, 0b00110, 0b01100, 0b01100, 0b01100, 0b01100,
        ],
        '8' => [
            0b11111, 0b11011, 0b11011, 0b11111, 0b11011, 0b11011, 0b11111,
        ],
        '9' => [
            0b11111, 0b11011, 0b11011, 0b11111, 0b00011, 0b00011, 0b11111,
        ],
        'A' => [
            0b00100, 0b01010, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        _ => [0; 7],
    }
}

/// Draws a face-down card inside `rect`.
pub fn draw_back(rect: Rect, highlighted: bool) {
    draw_back_with_theme(rect, highlighted, 0);
}

pub fn draw_back_with_theme(rect: Rect, highlighted: bool, theme: usize) {
    let theme = theme.min(5);
    let scale = (rect.h / CARD_HEIGHT).min(rect.w / CARD_WIDTH);
    let (navy, blue, gold) = match theme {
        1 => (
            Color::new(0.05, 0.25, 0.19, 1.0),
            Color::new(0.12, 0.52, 0.37, 1.0),
            Color::new(0.78, 0.92, 0.62, 1.0),
        ),
        2 => (
            Color::new(0.35, 0.11, 0.23, 1.0),
            Color::new(0.72, 0.24, 0.35, 1.0),
            Color::new(1.0, 0.78, 0.56, 1.0),
        ),
        3 => (
            Color::new(0.04, 0.07, 0.19, 1.0),
            Color::new(0.11, 0.19, 0.4, 1.0),
            Color::new(0.2, 0.9, 0.95, 1.0),
        ),
        4 => (
            Color::new(0.23, 0.15, 0.08, 1.0),
            Color::new(0.49, 0.33, 0.15, 1.0),
            Color::new(0.98, 0.83, 0.48, 1.0),
        ),
        5 => (
            Color::new(0.23, 0.08, 0.3, 1.0),
            Color::new(0.5, 0.2, 0.6, 1.0),
            Color::new(1.0, 0.73, 0.92, 1.0),
        ),
        _ => (
            Color::new(0.07, 0.17, 0.39, 1.0),
            Color::new(0.16, 0.38, 0.68, 1.0),
            Color::new(0.95, 0.73, 0.27, 1.0),
        ),
    };

    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if highlighted { GOLD } else { navy },
    );
    draw_rectangle(
        rect.x + 3.0 * scale,
        rect.y + 3.0 * scale,
        rect.w - 6.0 * scale,
        rect.h - 6.0 * scale,
        blue,
    );
    draw_rectangle_lines(
        rect.x + 7.0 * scale,
        rect.y + 7.0 * scale,
        rect.w - 14.0 * scale,
        rect.h - 14.0 * scale,
        2.0 * scale,
        gold,
    );

    match theme {
        1 => {
            for branch in 0..3 {
                let center_x = rect.x + rect.w / 2.0;
                let branch_y = rect.y + (25.0 + branch as f32 * 28.0) * scale;
                draw_line(
                    center_x,
                    branch_y - 10.0 * scale,
                    center_x,
                    branch_y + 10.0 * scale,
                    1.5 * scale,
                    gold,
                );
                for side in [-1.0, 1.0] {
                    draw_line(
                        center_x,
                        branch_y,
                        center_x + side * 13.0 * scale,
                        branch_y - 7.0 * scale,
                        1.5 * scale,
                        gold,
                    );
                    draw_ellipse(
                        center_x + side * 11.0 * scale - 3.0 * scale,
                        branch_y - 9.0 * scale,
                        6.0 * scale,
                        4.0 * scale,
                        0.0,
                        if side < 0.0 {
                            gold
                        } else {
                            Color::new(0.12, 0.52, 0.37, 1.0)
                        },
                    );
                }
            }
        }
        2 => {
            for row in 0..5 {
                for column in 0..3 {
                    draw_circle(
                        rect.x + (25.0 + column as f32 * 25.0) * scale,
                        rect.y + (18.0 + row as f32 * 27.0) * scale,
                        3.0 * scale,
                        if (row + column) % 2 == 0 {
                            gold
                        } else {
                            Color::new(0.98, 0.89, 0.78, 1.0)
                        },
                    );
                }
            }
        }
        3 => {
            for row in 0..5 {
                let y = rect.y + (18.0 + row as f32 * 25.0) * scale;
                let side = if row % 2 == 0 { 1.0 } else { -1.0 };
                draw_line(
                    rect.x + 12.0 * scale,
                    y,
                    rect.x + 30.0 * scale,
                    y,
                    2.0 * scale,
                    gold,
                );
                draw_line(
                    rect.x + 30.0 * scale,
                    y,
                    rect.x + 30.0 * scale,
                    y + side * 12.0 * scale,
                    2.0 * scale,
                    gold,
                );
                draw_circle(
                    rect.x + 30.0 * scale,
                    y,
                    2.5 * scale,
                    Color::new(0.6, 0.98, 1.0, 1.0),
                );
            }
        }
        4 => {
            for row in 0..4 {
                for column in 0..3 {
                    let center_x = rect.x + (19.0 + column as f32 * 25.0) * scale;
                    let center_y = rect.y + (25.0 + row as f32 * 30.0) * scale;
                    let size = 8.0 * scale;
                    draw_line(
                        center_x,
                        center_y - size,
                        center_x + size,
                        center_y,
                        1.5 * scale,
                        gold,
                    );
                    draw_line(
                        center_x + size,
                        center_y,
                        center_x,
                        center_y + size,
                        1.5 * scale,
                        gold,
                    );
                    draw_line(
                        center_x,
                        center_y + size,
                        center_x - size,
                        center_y,
                        1.5 * scale,
                        gold,
                    );
                    draw_line(
                        center_x - size,
                        center_y,
                        center_x,
                        center_y - size,
                        1.5 * scale,
                        gold,
                    );
                }
            }
        }
        5 => {
            for row in 0..5 {
                for column in 0..4 {
                    let center_x = rect.x + (17.0 + column as f32 * 22.0) * scale;
                    let center_y = rect.y + (20.0 + row as f32 * 25.0) * scale;
                    let radius = if (row + column) % 2 == 0 {
                        3.5 * scale
                    } else {
                        2.0 * scale
                    };
                    draw_circle(center_x, center_y, radius, gold);
                    draw_circle(
                        center_x,
                        center_y,
                        radius * 0.42,
                        Color::new(0.98, 0.85, 0.97, 1.0),
                    );
                }
            }
        }
        _ => {
            for row in 0..5 {
                for column in 0..3 {
                    let is_diamond = (row + column) % 2 == 0;
                    draw_pixel_pattern(
                        rect.x + (14.0 + column as f32 * 25.0) * scale,
                        rect.y + (19.0 + row as f32 * 25.0) * scale,
                        2.0 * scale,
                        suit_pattern(if is_diamond {
                            Suit::Diamonds
                        } else {
                            Suit::Hearts
                        }),
                        if is_diamond {
                            gold
                        } else {
                            Color::new(0.86, 0.89, 0.96, 1.0)
                        },
                    );
                }
            }
            draw_pixel_pattern(
                rect.x + rect.w / 2.0 - 14.0 * scale,
                rect.y + rect.h / 2.0 - 14.0 * scale,
                4.0 * scale,
                suit_pattern(Suit::Spades),
                navy,
            );
        }
    }

    match theme {
        1 => {
            let center_x = rect.x + rect.w / 2.0;
            let center_y = rect.y + rect.h / 2.0;
            draw_ellipse(
                center_x - 8.0 * scale,
                center_y - 13.0 * scale,
                10.0 * scale,
                19.0 * scale,
                0.0,
                gold,
            );
            draw_ellipse(
                center_x - 2.0 * scale,
                center_y - 13.0 * scale,
                10.0 * scale,
                19.0 * scale,
                0.0,
                blue,
            );
            draw_line(
                center_x,
                center_y - 10.0 * scale,
                center_x,
                center_y + 10.0 * scale,
                1.5 * scale,
                gold,
            );
        }
        2 => {
            let center_x = rect.x + rect.w / 2.0;
            let center_y = rect.y + rect.h / 2.0;
            draw_circle(center_x, center_y, 13.0 * scale, gold);
            draw_circle(
                center_x,
                center_y,
                9.0 * scale,
                Color::new(0.96, 0.4, 0.31, 1.0),
            );
            for angle in 0..8 {
                let radians = angle as f32 * std::f32::consts::PI / 4.0;
                let inner = 15.0 * scale;
                let outer = 20.0 * scale;
                draw_line(
                    center_x + radians.cos() * inner,
                    center_y + radians.sin() * inner,
                    center_x + radians.cos() * outer,
                    center_y + radians.sin() * outer,
                    1.5 * scale,
                    gold,
                );
            }
        }
        3 => {
            let center_x = rect.x + rect.w / 2.0;
            let center_y = rect.y + rect.h / 2.0;
            draw_circle(center_x, center_y, 14.0 * scale, navy);
            draw_circle(center_x, center_y, 10.0 * scale, gold);
            draw_circle(center_x, center_y, 5.0 * scale, navy);
            for angle in 0..4 {
                let radians = angle as f32 * std::f32::consts::PI / 2.0;
                draw_line(
                    center_x + radians.cos() * 16.0 * scale,
                    center_y + radians.sin() * 16.0 * scale,
                    center_x + radians.cos() * 23.0 * scale,
                    center_y + radians.sin() * 23.0 * scale,
                    2.0 * scale,
                    gold,
                );
            }
        }
        4 => {
            let center_x = rect.x + rect.w / 2.0;
            let center_y = rect.y + rect.h / 2.0;
            let radius = 18.0 * scale;
            for point in 0..8 {
                let angle = point as f32 * std::f32::consts::PI / 4.0;
                draw_circle(
                    center_x + angle.cos() * radius,
                    center_y + angle.sin() * radius,
                    3.0 * scale,
                    gold,
                );
            }
            draw_line(
                center_x - radius,
                center_y,
                center_x + radius,
                center_y,
                1.5 * scale,
                gold,
            );
            draw_line(
                center_x,
                center_y - radius,
                center_x,
                center_y + radius,
                1.5 * scale,
                gold,
            );
        }
        5 => {
            let center_x = rect.x + rect.w / 2.0;
            let center_y = rect.y + rect.h / 2.0;
            draw_circle(center_x, center_y, 17.0 * scale, gold);
            draw_circle(center_x, center_y, 12.0 * scale, blue);
            draw_circle(center_x, center_y, 7.0 * scale, gold);
            draw_line(
                center_x - 20.0 * scale,
                center_y - 20.0 * scale,
                center_x + 20.0 * scale,
                center_y + 20.0 * scale,
                2.0 * scale,
                gold,
            );
        }
        _ => {}
    }
}

/// Returns a compact rank-and-suit label such as `A S`.
pub fn format_card(card: Card) -> String {
    format!("{} {}", rank_label(card.rank), suit_letter(card.suit))
}

pub fn rank_label(rank: Rank) -> &'static str {
    match rank {
        Rank::Two => "2",
        Rank::Three => "3",
        Rank::Four => "4",
        Rank::Five => "5",
        Rank::Six => "6",
        Rank::Seven => "7",
        Rank::Eight => "8",
        Rank::Nine => "9",
        Rank::Ten => "10",
        Rank::Jack => "J",
        Rank::Queen => "Q",
        Rank::King => "K",
        Rank::Ace => "A",
    }
}

fn suit_letter(suit: Suit) -> &'static str {
    match suit {
        Suit::Clubs => "C",
        Suit::Diamonds => "D",
        Suit::Hearts => "H",
        Suit::Spades => "S",
    }
}

fn suit_color(suit: Suit) -> Color {
    match suit {
        Suit::Diamonds | Suit::Hearts => RED,
        Suit::Clubs | Suit::Spades => BLACK,
    }
}

fn pixel_glyph(character: char) -> [u8; 5] {
    match character {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'J' => [0b111, 0b001, 0b001, 0b101, 0b111],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'Q' => [0b111, 0b101, 0b101, 0b111, 0b001],
        _ => [0; 5],
    }
}

fn pixel_text_width(text: &str, pixel: f32) -> f32 {
    text.chars().count() as f32 * 3.0 * pixel
        + text.chars().count().saturating_sub(1) as f32 * pixel
}

fn draw_pixel_text(text: &str, x: f32, y: f32, pixel: f32, color: Color) {
    let mut cursor_x = x;
    for character in text.chars() {
        for (row, bits) in pixel_glyph(character).iter().enumerate() {
            for column in 0..3 {
                if bits & (1 << (2 - column)) != 0 {
                    draw_rectangle(
                        cursor_x + column as f32 * pixel,
                        y + row as f32 * pixel,
                        pixel,
                        pixel,
                        color,
                    );
                }
            }
        }
        cursor_x += pixel * 4.0;
    }
}

fn suit_pattern(suit: Suit) -> &'static [u8] {
    match suit {
        Suit::Clubs => &[
            0b0011100, 0b0111110, 0b1111111, 0b0011100, 0b0011100, 0b0111110, 0b0111110,
        ],
        Suit::Diamonds => &[
            0b0001000, 0b0011100, 0b0111110, 0b1111111, 0b0111110, 0b0011100, 0b0001000,
        ],
        Suit::Hearts => &[
            0b0110110, 0b1111111, 0b1111111, 0b1111111, 0b0111110, 0b0011100, 0b0001000,
        ],
        Suit::Spades => &[
            0b0001000, 0b0011100, 0b0111110, 0b1111111, 0b1111111, 0b0011100, 0b0111110,
        ],
    }
}

fn draw_pixel_pattern(x: f32, y: f32, pixel: f32, rows: &[u8], color: Color) {
    for (row, bits) in rows.iter().enumerate() {
        for column in 0..7 {
            if bits & (1 << (6 - column)) != 0 {
                draw_rectangle(
                    x + column as f32 * pixel,
                    y + row as f32 * pixel,
                    pixel,
                    pixel,
                    color,
                );
            }
        }
    }
}

fn draw_card_mascot(x: f32, y: f32, rank: Rank, scale: f32, theme: usize) {
    match theme {
        1 => draw_woodland_portrait(x, y, rank, scale),
        2 => draw_sunset_portrait(x, y, rank, scale),
        3 => draw_neon_portrait(x, y, rank, scale),
        4 => draw_royal_portrait(x, y, rank, scale),
        5 => draw_comic_portrait(x, y, rank, scale),
        _ => draw_classic_mascot(x, y, rank, scale),
    }
    draw_refined_face(x, y, rank, scale, theme);
}

fn draw_refined_face(x: f32, y: f32, rank: Rank, scale: f32, theme: usize) {
    let (skin, hair) = match theme {
        1 => (
            Color::new(0.94, 0.72, 0.51, 1.0),
            Color::new(0.24, 0.15, 0.11, 1.0),
        ),
        2 => (
            Color::new(1.0, 0.77, 0.59, 1.0),
            Color::new(0.38, 0.16, 0.29, 1.0),
        ),
        3 => (
            Color::new(0.87, 0.79, 0.69, 1.0),
            Color::new(0.08, 0.15, 0.28, 1.0),
        ),
        4 => (
            Color::new(0.91, 0.67, 0.43, 1.0),
            Color::new(0.3, 0.18, 0.11, 1.0),
        ),
        5 => (
            Color::new(1.0, 0.8, 0.48, 1.0),
            Color::new(0.27, 0.15, 0.17, 1.0),
        ),
        _ => (
            Color::new(1.0, 0.73, 0.43, 1.0),
            Color::new(0.19, 0.13, 0.1, 1.0),
        ),
    };
    let ink = Color::new(0.16, 0.12, 0.12, 1.0);
    let face_x = x + 8.5 * scale;
    let face_y = y + 13.0 * scale;
    let accent = match theme {
        1 => Color::new(0.76, 0.6, 0.25, 1.0),
        2 => Color::new(0.94, 0.47, 0.27, 1.0),
        3 => Color::new(0.13, 0.9, 0.96, 1.0),
        4 => Color::new(0.85, 0.63, 0.22, 1.0),
        5 => Color::new(0.96, 0.69, 0.2, 1.0),
        _ => GOLD,
    };

    draw_ellipse(
        x + 5.0 * scale,
        y + 9.0 * scale,
        25.0 * scale,
        34.0 * scale,
        0.0,
        hair,
    );
    draw_ellipse(face_x, face_y, 18.0 * scale, 26.0 * scale, 0.0, skin);

    for eye_x in [13.5, 21.5] {
        draw_line(
            x + (eye_x - 2.0) * scale,
            y + 21.0 * scale,
            x + (eye_x + 1.5) * scale,
            y + 20.5 * scale,
            1.0 * scale,
            hair,
        );
        draw_ellipse(
            x + (eye_x - 0.8) * scale,
            y + 23.0 * scale,
            1.8 * scale,
            1.4 * scale,
            0.0,
            ink,
        );
        draw_circle(
            x + (eye_x + 0.2) * scale,
            y + 23.2 * scale,
            0.35 * scale,
            WHITE,
        );
    }

    draw_line(
        x + 17.5 * scale,
        y + 25.0 * scale,
        x + 16.8 * scale,
        y + 29.0 * scale,
        0.9 * scale,
        Color::new(0.68, 0.4, 0.3, 1.0),
    );

    match rank {
        Rank::King => {
            let beard = match theme {
                3 => Color::new(0.11, 0.31, 0.39, 1.0),
                4 => Color::new(0.25, 0.15, 0.1, 1.0),
                _ => hair,
            };
            draw_triangle(
                vec2(x + 10.0 * scale, y + 31.0 * scale),
                vec2(x + 25.0 * scale, y + 31.0 * scale),
                vec2(x + 17.5 * scale, y + 41.0 * scale),
                beard,
            );
            draw_line(
                x + 14.5 * scale,
                y + 31.5 * scale,
                x + 20.5 * scale,
                y + 31.5 * scale,
                1.0 * scale,
                Color::new(0.9, 0.63, 0.48, 1.0),
            );
        }
        Rank::Queen => {
            draw_ellipse(
                x + 14.5 * scale,
                y + 32.0 * scale,
                6.0 * scale,
                1.4 * scale,
                0.0,
                Color::new(0.67, 0.24, 0.31, 1.0),
            );
            for cheek_x in [11.0, 24.0] {
                draw_ellipse(
                    x + cheek_x * scale,
                    y + 29.0 * scale,
                    2.0 * scale,
                    1.0 * scale,
                    0.0,
                    Color::new(0.94, 0.48, 0.44, 0.55),
                );
            }
        }
        _ => {
            draw_line(
                x + 14.5 * scale,
                y + 32.0 * scale,
                x + 17.5 * scale,
                y + 33.0 * scale,
                1.0 * scale,
                ink,
            );
            draw_line(
                x + 17.5 * scale,
                y + 33.0 * scale,
                x + 20.5 * scale,
                y + 32.0 * scale,
                1.0 * scale,
                ink,
            );
        }
    }

    match rank {
        Rank::Jack => {
            let cap = match theme {
                3 => Color::new(0.12, 0.28, 0.44, 1.0),
                5 => Color::new(0.17, 0.47, 0.67, 1.0),
                _ => Color::new(0.2, 0.36, 0.48, 1.0),
            };
            draw_ellipse(
                x + 5.0 * scale,
                y + 10.0 * scale,
                25.0 * scale,
                7.0 * scale,
                0.0,
                cap,
            );
            draw_triangle(
                vec2(x + 19.0 * scale, y + 12.0 * scale),
                vec2(x + 29.0 * scale, y + 3.0 * scale),
                vec2(x + 27.0 * scale, y + 14.0 * scale),
                accent,
            );
            draw_circle(x + 9.0 * scale, y + 13.0 * scale, 1.2 * scale, accent);
        }
        Rank::Queen | Rank::King => {
            let base_y = if rank == Rank::Queen { 12.0 } else { 13.0 };
            draw_rectangle(
                x + 6.0 * scale,
                y + base_y * scale,
                23.0 * scale,
                3.0 * scale,
                accent,
            );
            let peaks: &[(f32, f32)] = if rank == Rank::Queen {
                &[(7.0, 5.0), (14.0, 1.0), (21.0, 5.0)]
            } else {
                &[
                    (6.0, 4.0),
                    (12.0, 1.0),
                    (17.5, 3.0),
                    (23.0, 1.0),
                    (29.0, 4.0),
                ]
            };
            for (offset, peak) in peaks {
                draw_triangle(
                    vec2(x + (offset - 1.5) * scale, y + base_y * scale),
                    vec2(x + *offset * scale, y + *peak * scale),
                    vec2(x + (offset + 1.5) * scale, y + base_y * scale),
                    accent,
                );
                draw_circle(
                    x + *offset * scale,
                    y + (*peak + 2.0) * scale,
                    0.8 * scale,
                    if theme == 3 {
                        WHITE
                    } else {
                        Color::new(0.76, 0.2, 0.26, 1.0)
                    },
                );
            }
        }
        _ => {}
    }
}

fn draw_neon_portrait(x: f32, y: f32, rank: Rank, scale: f32) {
    let cyan = Color::new(0.13, 0.9, 0.96, 1.0);
    let pink = Color::new(0.95, 0.24, 0.71, 1.0);
    let dark = Color::new(0.1, 0.16, 0.35, 1.0);
    draw_rectangle(
        x + 5.0 * scale,
        y + 15.0 * scale,
        25.0 * scale,
        24.0 * scale,
        dark,
    );
    draw_rectangle_lines(
        x + 5.0 * scale,
        y + 15.0 * scale,
        25.0 * scale,
        24.0 * scale,
        2.0 * scale,
        cyan,
    );
    for eye_x in [12.0, 23.0] {
        draw_rectangle(
            x + eye_x * scale,
            y + 23.0 * scale,
            3.0 * scale,
            4.0 * scale,
            pink,
        );
    }
    draw_line(
        x + 12.0 * scale,
        y + 33.0 * scale,
        x + 24.0 * scale,
        y + 33.0 * scale,
        2.0 * scale,
        cyan,
    );
    draw_rectangle(
        x + 16.0 * scale,
        y + 39.0 * scale,
        4.0 * scale,
        8.0 * scale,
        cyan,
    );
    draw_triangle(
        vec2(x + 2.0 * scale, y + 64.0 * scale),
        vec2(x + 17.0 * scale, y + 44.0 * scale),
        vec2(x + 33.0 * scale, y + 64.0 * scale),
        dark,
    );
    draw_line(
        x + 17.0 * scale,
        y + 48.0 * scale,
        x + 17.0 * scale,
        y + 62.0 * scale,
        2.0 * scale,
        pink,
    );
    for spark in 0..3 {
        let spark_y = (10.0 + spark as f32 * 15.0) * scale;
        draw_line(
            x + 1.0 * scale,
            y + spark_y,
            x + 4.0 * scale,
            y + spark_y,
            scale,
            cyan,
        );
        draw_line(
            x + 31.0 * scale,
            y + spark_y + 5.0 * scale,
            x + 34.0 * scale,
            y + spark_y + 5.0 * scale,
            scale,
            pink,
        );
    }
    if rank == Rank::King {
        draw_line(
            x + 8.0 * scale,
            y + 13.0 * scale,
            x + 17.0 * scale,
            y + 2.0 * scale,
            2.0 * scale,
            pink,
        );
        draw_line(
            x + 17.0 * scale,
            y + 2.0 * scale,
            x + 27.0 * scale,
            y + 13.0 * scale,
            2.0 * scale,
            pink,
        );
    }
}

fn draw_royal_portrait(x: f32, y: f32, rank: Rank, scale: f32) {
    let gold = Color::new(0.76, 0.48, 0.12, 1.0);
    let robe = match rank {
        Rank::Jack => Color::new(0.21, 0.34, 0.62, 1.0),
        Rank::Queen => Color::new(0.58, 0.19, 0.4, 1.0),
        Rank::King => Color::new(0.37, 0.2, 0.49, 1.0),
        _ => Color::new(0.35, 0.3, 0.18, 1.0),
    };
    let skin = Color::new(0.94, 0.68, 0.43, 1.0);
    draw_triangle(
        vec2(x + 2.0 * scale, y + 64.0 * scale),
        vec2(x + 17.5 * scale, y + 39.0 * scale),
        vec2(x + 33.0 * scale, y + 64.0 * scale),
        robe,
    );
    draw_rectangle(
        x + 15.0 * scale,
        y + 34.0 * scale,
        5.0 * scale,
        9.0 * scale,
        skin,
    );
    draw_circle(x + 17.5 * scale, y + 24.0 * scale, 11.0 * scale, skin);
    draw_rectangle(
        x + 12.0 * scale,
        y + 23.0 * scale,
        3.0 * scale,
        2.0 * scale,
        INK,
    );
    draw_rectangle(
        x + 21.0 * scale,
        y + 23.0 * scale,
        3.0 * scale,
        2.0 * scale,
        INK,
    );
    draw_line(
        x + 14.0 * scale,
        y + 31.0 * scale,
        x + 21.0 * scale,
        y + 31.0 * scale,
        1.5 * scale,
        Color::new(0.55, 0.2, 0.17, 1.0),
    );
    draw_rectangle(
        x + 16.0 * scale,
        y + 43.0 * scale,
        3.0 * scale,
        17.0 * scale,
        gold,
    );
    draw_circle(
        x + 17.5 * scale,
        y + 51.0 * scale,
        3.5 * scale,
        Color::new(0.2, 0.62, 0.72, 1.0),
    );
    for point in 0..5 {
        let crown_x = (7.0 + point as f32 * 5.2) * scale;
        let peak = if point % 2 == 0 { 4.0 } else { 8.0 };
        draw_triangle(
            vec2(x + crown_x, y + 16.0 * scale),
            vec2(x + (crown_x / scale + 2.6) * scale, y + peak * scale),
            vec2(x + (crown_x / scale + 5.2) * scale, y + 16.0 * scale),
            gold,
        );
    }
    if rank == Rank::Queen {
        draw_circle(
            x + 17.5 * scale,
            y + 10.0 * scale,
            2.0 * scale,
            Color::new(0.76, 0.2, 0.26, 1.0),
        );
    }
}

fn draw_comic_portrait(x: f32, y: f32, rank: Rank, scale: f32) {
    let outline = Color::new(0.16, 0.12, 0.2, 1.0);
    let face = Color::new(1.0, 0.79, 0.35, 1.0);
    let robe = match rank {
        Rank::Jack => Color::new(0.15, 0.62, 0.78, 1.0),
        Rank::Queen => Color::new(0.88, 0.25, 0.47, 1.0),
        Rank::King => Color::new(0.37, 0.35, 0.79, 1.0),
        _ => Color::new(0.88, 0.45, 0.15, 1.0),
    };
    draw_triangle(
        vec2(x + 1.0 * scale, y + 64.0 * scale),
        vec2(x + 17.5 * scale, y + 38.0 * scale),
        vec2(x + 34.0 * scale, y + 64.0 * scale),
        outline,
    );
    draw_triangle(
        vec2(x + 4.0 * scale, y + 62.0 * scale),
        vec2(x + 17.5 * scale, y + 42.0 * scale),
        vec2(x + 31.0 * scale, y + 62.0 * scale),
        robe,
    );
    draw_circle(x + 17.5 * scale, y + 25.0 * scale, 14.0 * scale, outline);
    draw_circle(x + 17.5 * scale, y + 25.0 * scale, 11.0 * scale, face);
    draw_ellipse(
        x + 8.0 * scale,
        y + 15.0 * scale,
        18.0 * scale,
        7.0 * scale,
        0.0,
        outline,
    );
    for eye_x in [12.5, 22.5] {
        draw_circle(x + eye_x * scale, y + 24.0 * scale, 2.4 * scale, WHITE);
        draw_circle(x + eye_x * scale, y + 24.0 * scale, scale, outline);
    }
    draw_line(
        x + 13.0 * scale,
        y + 32.0 * scale,
        x + 22.0 * scale,
        y + 32.0 * scale,
        2.0 * scale,
        outline,
    );
    match rank {
        Rank::Jack => {
            draw_line(
                x + 4.0 * scale,
                y + 14.0 * scale,
                x + 29.0 * scale,
                y + 12.0 * scale,
                3.0 * scale,
                robe,
            );
            draw_triangle(
                vec2(x + 25.0 * scale, y + 12.0 * scale),
                vec2(x + 32.0 * scale, y + 8.0 * scale),
                vec2(x + 30.0 * scale, y + 16.0 * scale),
                face,
            );
        }
        Rank::Queen => {
            draw_circle(
                x + 9.0 * scale,
                y + 12.0 * scale,
                3.0 * scale,
                Color::new(0.94, 0.72, 0.13, 1.0),
            );
            draw_circle(
                x + 17.5 * scale,
                y + 8.0 * scale,
                3.0 * scale,
                Color::new(0.94, 0.72, 0.13, 1.0),
            );
            draw_circle(
                x + 26.0 * scale,
                y + 12.0 * scale,
                3.0 * scale,
                Color::new(0.94, 0.72, 0.13, 1.0),
            );
        }
        Rank::King => {
            draw_line(
                x + 7.0 * scale,
                y + 13.0 * scale,
                x + 8.0 * scale,
                y + 4.0 * scale,
                3.0 * scale,
                Color::new(0.94, 0.72, 0.13, 1.0),
            );
            draw_line(
                x + 8.0 * scale,
                y + 4.0 * scale,
                x + 17.0 * scale,
                y + 12.0 * scale,
                3.0 * scale,
                Color::new(0.94, 0.72, 0.13, 1.0),
            );
            draw_line(
                x + 17.0 * scale,
                y + 12.0 * scale,
                x + 27.0 * scale,
                y + 3.0 * scale,
                3.0 * scale,
                Color::new(0.94, 0.72, 0.13, 1.0),
            );
            draw_line(
                x + 27.0 * scale,
                y + 3.0 * scale,
                x + 29.0 * scale,
                y + 14.0 * scale,
                3.0 * scale,
                Color::new(0.94, 0.72, 0.13, 1.0),
            );
        }
        _ => {}
    }
}

fn draw_woodland_portrait(x: f32, y: f32, rank: Rank, scale: f32) {
    let skin = Color::new(0.94, 0.66, 0.39, 1.0);
    let hair = Color::new(0.28, 0.15, 0.09, 1.0);
    let robe = match rank {
        Rank::Jack => Color::new(0.19, 0.42, 0.27, 1.0),
        Rank::Queen => Color::new(0.55, 0.27, 0.38, 1.0),
        Rank::King => Color::new(0.25, 0.32, 0.52, 1.0),
        _ => Color::new(0.28, 0.44, 0.29, 1.0),
    };
    draw_triangle(
        vec2(x + 2.0 * scale, y + 63.0 * scale),
        vec2(x + 17.5 * scale, y + 35.0 * scale),
        vec2(x + 33.0 * scale, y + 63.0 * scale),
        robe,
    );
    draw_rectangle(
        x + 15.0 * scale,
        y + 31.0 * scale,
        5.0 * scale,
        11.0 * scale,
        skin,
    );
    draw_ellipse(
        x + 6.0 * scale,
        y + 11.0 * scale,
        23.0 * scale,
        29.0 * scale,
        0.0,
        hair,
    );
    draw_ellipse(
        x + 9.0 * scale,
        y + 15.0 * scale,
        17.0 * scale,
        23.0 * scale,
        0.0,
        skin,
    );
    draw_rectangle(
        x + 12.0 * scale,
        y + 23.0 * scale,
        3.0 * scale,
        2.0 * scale,
        INK,
    );
    draw_rectangle(
        x + 20.0 * scale,
        y + 23.0 * scale,
        3.0 * scale,
        2.0 * scale,
        INK,
    );
    draw_line(
        x + 14.0 * scale,
        y + 32.0 * scale,
        x + 21.0 * scale,
        y + 32.0 * scale,
        1.5 * scale,
        Color::new(0.48, 0.18, 0.12, 1.0),
    );
    draw_line(
        x + 17.5 * scale,
        y + 42.0 * scale,
        x + 17.5 * scale,
        y + 59.0 * scale,
        2.0 * scale,
        GOLD,
    );

    match rank {
        Rank::Jack => {
            draw_rectangle(
                x + 4.0 * scale,
                y + 13.0 * scale,
                27.0 * scale,
                4.0 * scale,
                robe,
            );
            draw_triangle(
                vec2(x + 14.0 * scale, y + 14.0 * scale),
                vec2(x + 25.0 * scale, y + 3.0 * scale),
                vec2(x + 23.0 * scale, y + 16.0 * scale),
                Color::new(0.31, 0.57, 0.27, 1.0),
            );
        }
        Rank::Queen => {
            draw_triangle(
                vec2(x + 7.0 * scale, y + 16.0 * scale),
                vec2(x + 17.5 * scale, y - 1.0 * scale),
                vec2(x + 28.0 * scale, y + 16.0 * scale),
                GOLD,
            );
            for jewel_x in [12.0, 17.0, 22.0] {
                draw_circle(
                    x + jewel_x * scale,
                    y + 10.0 * scale,
                    1.6 * scale,
                    Color::new(0.65, 0.15, 0.2, 1.0),
                );
            }
        }
        Rank::King => {
            draw_triangle(
                vec2(x + 7.0 * scale, y + 16.0 * scale),
                vec2(x + 7.0 * scale, y + 4.0 * scale),
                vec2(x + 13.0 * scale, y + 12.0 * scale),
                GOLD,
            );
            draw_triangle(
                vec2(x + 13.0 * scale, y + 12.0 * scale),
                vec2(x + 18.0 * scale, y + 2.0 * scale),
                vec2(x + 22.0 * scale, y + 12.0 * scale),
                GOLD,
            );
            draw_triangle(
                vec2(x + 22.0 * scale, y + 12.0 * scale),
                vec2(x + 29.0 * scale, y + 4.0 * scale),
                vec2(x + 28.0 * scale, y + 16.0 * scale),
                GOLD,
            );
            draw_triangle(
                vec2(x + 10.0 * scale, y + 29.0 * scale),
                vec2(x + 25.0 * scale, y + 29.0 * scale),
                vec2(x + 17.5 * scale, y + 40.0 * scale),
                hair,
            );
        }
        _ => {
            draw_circle(
                x + 18.0 * scale,
                y + 10.0 * scale,
                7.0 * scale,
                Color::new(0.29, 0.52, 0.28, 1.0),
            );
        }
    }
}

fn draw_sunset_portrait(x: f32, y: f32, rank: Rank, scale: f32) {
    let skin = Color::new(1.0, 0.77, 0.57, 1.0);
    let plum = Color::new(0.47, 0.19, 0.39, 1.0);
    let sun = Color::new(0.98, 0.53, 0.28, 1.0);
    let robe = match rank {
        Rank::Jack => Color::new(0.18, 0.42, 0.57, 1.0),
        Rank::Queen => Color::new(0.65, 0.24, 0.42, 1.0),
        Rank::King => Color::new(0.38, 0.22, 0.53, 1.0),
        _ => sun,
    };
    draw_circle(
        x + 17.5 * scale,
        y + 25.0 * scale,
        19.0 * scale,
        Color::new(0.98, 0.74, 0.42, 1.0),
    );
    draw_triangle(
        vec2(x + 1.0 * scale, y + 63.0 * scale),
        vec2(x + 17.5 * scale, y + 37.0 * scale),
        vec2(x + 34.0 * scale, y + 63.0 * scale),
        robe,
    );
    draw_rectangle(
        x + 15.0 * scale,
        y + 34.0 * scale,
        5.0 * scale,
        8.0 * scale,
        skin,
    );
    draw_circle(x + 17.5 * scale, y + 24.0 * scale, 12.0 * scale, plum);
    draw_circle(x + 17.5 * scale, y + 26.0 * scale, 9.0 * scale, skin);
    draw_circle(
        x + 13.0 * scale,
        y + 24.0 * scale,
        1.2 * scale,
        Color::new(0.2, 0.1, 0.19, 1.0),
    );
    draw_circle(
        x + 22.0 * scale,
        y + 24.0 * scale,
        1.2 * scale,
        Color::new(0.2, 0.1, 0.19, 1.0),
    );
    draw_line(
        x + 14.0 * scale,
        y + 31.0 * scale,
        x + 21.0 * scale,
        y + 31.0 * scale,
        1.5 * scale,
        Color::new(0.7, 0.24, 0.25, 1.0),
    );
    match rank {
        Rank::Jack => {
            draw_rectangle(
                x + 5.0 * scale,
                y + 11.0 * scale,
                25.0 * scale,
                3.0 * scale,
                Color::new(0.2, 0.45, 0.57, 1.0),
            );
            draw_triangle(
                vec2(x + 13.0 * scale, y + 11.0 * scale),
                vec2(x + 22.0 * scale, y + 1.0 * scale),
                vec2(x + 23.0 * scale, y + 13.0 * scale),
                sun,
            );
        }
        Rank::Queen => {
            draw_circle(x + 9.0 * scale, y + 14.0 * scale, 3.0 * scale, sun);
            draw_circle(x + 17.5 * scale, y + 11.0 * scale, 3.0 * scale, sun);
            draw_circle(x + 26.0 * scale, y + 14.0 * scale, 3.0 * scale, sun);
        }
        Rank::King => {
            draw_triangle(
                vec2(x + 8.0 * scale, y + 14.0 * scale),
                vec2(x + 12.0 * scale, y + 3.0 * scale),
                vec2(x + 16.0 * scale, y + 14.0 * scale),
                sun,
            );
            draw_triangle(
                vec2(x + 15.0 * scale, y + 14.0 * scale),
                vec2(x + 20.0 * scale, y + 1.0 * scale),
                vec2(x + 24.0 * scale, y + 14.0 * scale),
                sun,
            );
            draw_triangle(
                vec2(x + 12.0 * scale, y + 32.0 * scale),
                vec2(x + 23.0 * scale, y + 32.0 * scale),
                vec2(x + 17.5 * scale, y + 40.0 * scale),
                plum,
            );
        }
        _ => {
            draw_circle(x + 17.5 * scale, y + 9.0 * scale, 5.0 * scale, sun);
        }
    }
}

fn draw_classic_mascot(x: f32, y: f32, rank: Rank, scale: f32) {
    macro_rules! pattern {
        ($dx:expr, $dy:expr, $pixel:expr, $rows:expr, $color:expr $(,)?) => {
            draw_pixel_pattern(
                x + $dx * scale,
                y + $dy * scale,
                $pixel * scale,
                $rows,
                $color,
            )
        };
    }
    macro_rules! rect {
        ($dx:expr, $dy:expr, $width:expr, $height:expr, $color:expr) => {
            draw_rectangle(
                x + $dx * scale,
                y + $dy * scale,
                $width * scale,
                $height * scale,
                $color,
            )
        };
    }
    let ink = Color::new(0.1, 0.13, 0.17, 1.0);
    let skin = Color::new(1.0, 0.69, 0.31, 1.0);

    match rank {
        Rank::Ace => {
            pattern!(
                3.0,
                2.0,
                4.0,
                &[
                    0b0001000, 0b0011100, 0b1111111, 0b0111110, 0b0111110, 0b0011100, 0b0011100,
                ],
                GOLD,
            );
            rect!(14.0, 16.0, 4.0, 4.0, WHITE);
        }
        Rank::Jack => {
            let tunic = Color::new(0.15, 0.44, 0.72, 1.0);
            draw_face_body(x, y, scale, tunic, GOLD);
            pattern!(3.0, 11.0, 4.0, &[0b0011100, 0b0111110], ink);
            pattern!(
                3.0,
                19.0,
                4.0,
                &[0b0111110, 0b1111111, 0b1111111, 0b1111111, 0b0111110],
                skin,
            );
            rect!(10.0, 27.0, 4.0, 4.0, ink);
            rect!(22.0, 27.0, 4.0, 4.0, ink);
            pattern!(3.0, 0.0, 4.0, &[0b0010000, 0b0111000, 0b1111111], GOLD);
            pattern!(11.0, 37.0, 4.0, &[0b0001000, 0b0011100], skin);
            rect!(15.0, 47.0, 4.0, 16.0, GOLD);
        }
        Rank::Queen => {
            let hair = Color::new(0.55, 0.16, 0.39, 1.0);
            draw_face_body(x, y, scale, hair, GOLD);
            pattern!(
                3.0,
                9.0,
                4.0,
                &[
                    0b0111110, 0b1111111, 0b1111111, 0b1111111, 0b0111110, 0b0111110,
                ],
                hair,
            );
            pattern!(
                7.0,
                17.0,
                4.0,
                &[0b0011100, 0b0111110, 0b0111110, 0b0111110],
                skin,
            );
            rect!(11.0, 29.0, 4.0, 4.0, ink);
            rect!(21.0, 29.0, 4.0, 4.0, ink);
            pattern!(11.0, 38.0, 4.0, &[0b0111110, 0b0011100], PINK);
            pattern!(3.0, 0.0, 4.0, &[0b1010101, 0b1111111], GOLD);
            pattern!(14.0, 47.0, 4.0, &[0b0011100, 0b0011100], GOLD);
        }
        Rank::King => {
            let beard = Color::new(0.42, 0.22, 0.11, 1.0);
            let robe = Color::new(0.43, 0.2, 0.58, 1.0);
            draw_face_body(x, y, scale, robe, GOLD);
            pattern!(
                3.0,
                13.0,
                4.0,
                &[0b0111110, 0b1111111, 0b1111111, 0b1111111, 0b0111110],
                skin,
            );
            rect!(10.0, 25.0, 4.0, 4.0, ink);
            rect!(22.0, 25.0, 4.0, 4.0, ink);
            pattern!(
                7.0,
                32.0,
                4.0,
                &[0b0111110, 0b0111110, 0b0011100, 0b0011100],
                beard,
            );
            pattern!(3.0, 0.0, 4.0, &[0b1010101, 0b1111111, 0b1111111], GOLD);
            pattern!(15.0, 49.0, 4.0, &[0b0011100, 0b0011100], GOLD);
        }
        _ => (),
    }
}

fn draw_face_body(x: f32, y: f32, scale: f32, robe: Color, trim: Color) {
    draw_pixel_pattern(
        x,
        y + 34.0 * scale,
        4.0 * scale,
        &[
            0b0001000, 0b0011100, 0b0111110, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111,
        ],
        robe,
    );
    draw_pixel_pattern(
        x + 11.0 * scale,
        y + 30.0 * scale,
        4.0 * scale,
        &[
            0b0001000, 0b0011100, 0b0011100, 0b0011100, 0b0011100, 0b0011100,
        ],
        Color::new(1.0, 0.69, 0.31, 1.0),
    );
    draw_rectangle(
        x + 13.0 * scale,
        y + 44.0 * scale,
        4.0 * scale,
        16.0 * scale,
        trim,
    );
}

fn has_face_portrait(rank: Rank) -> bool {
    matches!(rank, Rank::Jack | Rank::Queen | Rank::King)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_font_has_distinct_rank_glyphs() {
        for glyph in [
            '2', '3', '4', '5', '6', '7', '8', '9', '1', '0', 'A', 'J', 'Q', 'K',
        ] {
            assert_ne!(pixel_glyph(glyph), [0; 5]);
        }
        assert!(pixel_text_width("10", 5.0) > pixel_text_width("A", 5.0));
    }

    #[test]
    fn card_front_themes_use_distinct_number_fonts() {
        for glyph in '0'..='9' {
            let variants = (1..5)
                .map(|theme| themed_glyph(glyph, theme))
                .collect::<std::collections::HashSet<_>>();
            assert_eq!(variants.len(), 4, "glyph {glyph}");
            assert!(variants.iter().all(|variant| *variant != [0; 7]));
            assert_ne!(themed_glyph(glyph, 5), [0; 7]);
        }
        assert!(rank_glyph_width("10", 1) > rank_glyph_width("7", 1));
        assert!(rank_glyph_width("10", 2) > rank_glyph_width("7", 2));
    }

    #[test]
    fn only_jack_queen_and_king_have_face_portraits() {
        for rank in [Rank::Jack, Rank::Queen, Rank::King] {
            assert!(has_face_portrait(rank));
        }
        for rank in [
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Ace,
        ] {
            assert!(!has_face_portrait(rank));
        }
    }
}
