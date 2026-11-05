#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, process};

use macroquad::prelude::*;
use vandtia::card_renderer::{self, CARD_HEIGHT, CARD_WIDTH, CardStyle};
use vandtia::games::{
    sjuan, solitaire,
    vandtia::{Game, PlayerPhase, TurnOutcome},
};
use vandtia::{Card, Rank, Suit, simulator::play_computer_turn};

#[path = "graphics/progress.rs"]
mod progress;

const CARD_GAP: f32 = 14.0;
const ACTION_BUTTON_HEIGHT: f32 = 38.0;
const COMPACT_ACTION_BUTTON_HEIGHT: f32 = 58.0;
const NEXT_TURN_DELAY: f64 = 0.7;
const INITIAL_STATUS: &str = "Markera kort och klicka på SPELA MARKERADE eller spelhögen.";
const TABLE_COLOR: Color = Color::new(0.05, 0.28, 0.19, 1.0);
const PANEL_COLOR: Color = Color::new(0.02, 0.12, 0.09, 1.0);

fn window_conf() -> Conf {
    Conf {
        window_title: "Kortspel".to_owned(),
        window_width: 1100,
        window_height: 760,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("Fel: {error}");
        process::exit(2);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (mut players, seed) = parse_args()?;
    let mut progress = progress::load()?;
    let mut game_session = None;
    let mut solitaire = None;
    let mut sjuan = None;
    let mut solitaire_rewarded = false;
    let mut sjuan_rewarded = false;
    let mut vandtia_rewarded = false;
    let mut sjuan_cursor = 0usize;
    let mut sjuan_status = "Lägg en sjua först. Bygg sedan vidare i samma färg.".to_owned();
    let mut shop_status = "Välj en stil för att köpa eller utrusta den.".to_owned();
    let mut show_card_gallery = false;
    let mut show_shop = false;
    let mut shop_front_tab = true;
    let mut card_gallery_size = 1usize;
    let mut show_menu = true;
    let mut cursor = 0usize;
    let mut selected = Vec::new();
    let mut status = INITIAL_STATUS.to_owned();
    let mut ai_next_at = 0.0;

    loop {
        if show_menu {
            draw_main_menu(screen_width(), screen_height(), &progress);
            if is_key_pressed(KeyCode::Key1) {
                show_menu = false;
            } else if is_key_pressed(KeyCode::Key2) {
                sjuan = Some(sjuan::Sjuan::new(players, seed_or_random(seed)?)?);
                sjuan_rewarded = false;
                sjuan_cursor = 0;
                ai_next_at = 0.0;
                show_menu = false;
            } else if is_key_pressed(KeyCode::Key3) {
                solitaire = Some(solitaire::Solitaire::new(seed_or_random(seed)?));
                solitaire_rewarded = false;
                show_menu = false;
            } else if is_key_pressed(KeyCode::Key4) {
                show_shop = true;
                show_menu = false;
            } else if is_key_pressed(KeyCode::Key5) {
                show_card_gallery = true;
                show_menu = false;
            } else if is_key_pressed(KeyCode::Q) {
                break;
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                let (x, y) = mouse_position();
                if draw_button_contains(vandtia_menu_button(screen_width(), screen_height()), x, y)
                {
                    show_menu = false;
                } else if draw_button_contains(
                    sjuan_menu_option_rect(screen_width(), screen_height()),
                    x,
                    y,
                ) {
                    sjuan = Some(sjuan::Sjuan::new(players, seed_or_random(seed)?)?);
                    sjuan_rewarded = false;
                    sjuan_cursor = 0;
                    ai_next_at = 0.0;
                    show_menu = false;
                } else if draw_button_contains(
                    solitaire_menu_option_rect(screen_width(), screen_height()),
                    x,
                    y,
                ) {
                    solitaire = Some(solitaire::Solitaire::new(seed_or_random(seed)?));
                    solitaire_rewarded = false;
                    show_menu = false;
                } else if draw_button_contains(
                    shop_menu_option_rect(screen_width(), screen_height()),
                    x,
                    y,
                ) {
                    show_shop = true;
                    show_menu = false;
                } else if draw_button_contains(
                    card_gallery_menu_option_rect(screen_width(), screen_height()),
                    x,
                    y,
                ) {
                    show_card_gallery = true;
                    show_menu = false;
                }
            }
            next_frame().await;
            continue;
        }

        if show_shop {
            draw_shop(
                screen_width(),
                screen_height(),
                &progress,
                shop_front_tab,
                &shop_status,
            );
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::M) {
                show_shop = false;
                show_menu = true;
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                let (x, y) = mouse_position();
                if draw_button_contains(
                    card_gallery_menu_button(screen_width(), screen_height()),
                    x,
                    y,
                ) {
                    show_shop = false;
                    show_menu = true;
                } else if let Some(tab) = shop_tab_at(x, y, screen_width()) {
                    shop_front_tab = tab;
                } else if let Some(theme) = shop_choice_at(x, y, screen_width(), screen_height()) {
                    let result = if shop_front_tab {
                        progress.buy_or_equip_front(theme)
                    } else {
                        progress.buy_or_equip_back(theme)
                    };
                    match result {
                        Ok(true) => {
                            shop_status = if shop_front_tab {
                                format!("Framsidans stil {} är vald.", theme + 1)
                            } else {
                                format!("Baksidans stil {} är vald.", theme + 1)
                            };
                            progress::save(&progress)?;
                        }
                        Ok(false) => {}
                        Err(error) => shop_status = error.to_owned(),
                    }
                }
            }
            next_frame().await;
            continue;
        }

        if show_card_gallery {
            draw_card_gallery(
                screen_width(),
                screen_height(),
                card_gallery_size,
                &progress,
            );
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::M) {
                show_card_gallery = false;
                show_menu = true;
            } else if is_key_pressed(KeyCode::Key1) {
                card_gallery_size = 0;
            } else if is_key_pressed(KeyCode::Key2) {
                card_gallery_size = 1;
            } else if is_key_pressed(KeyCode::Key3) {
                card_gallery_size = 2;
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                let (x, y) = mouse_position();
                if draw_button_contains(
                    card_gallery_menu_button(screen_width(), screen_height()),
                    x,
                    y,
                ) {
                    show_card_gallery = false;
                    show_menu = true;
                } else {
                    for (size, rect) in card_gallery_size_buttons(screen_width(), screen_height())
                        .into_iter()
                        .enumerate()
                    {
                        if draw_button_contains(rect, x, y) {
                            card_gallery_size = size;
                        }
                    }
                }
            }
            next_frame().await;
            continue;
        }

        if let Some(game) = solitaire.as_mut() {
            draw_solitaire(game, screen_width(), screen_height(), &progress);
            if game.has_won() && !solitaire_rewarded {
                progress.award(15);
                progress::save(&progress)?;
                solitaire_rewarded = true;
            }
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::M) {
                solitaire = None;
                show_menu = true;
            } else if is_key_pressed(KeyCode::N) {
                solitaire = Some(solitaire::Solitaire::new(seed_or_random(seed)?));
                solitaire_rewarded = false;
            } else if is_mouse_button_pressed(MouseButton::Left) {
                let (x, y) = mouse_position();
                if draw_button_contains(
                    solitaire_menu_button(screen_width(), screen_height()),
                    x,
                    y,
                ) {
                    solitaire = None;
                    show_menu = true;
                } else {
                    handle_solitaire_click(game, x, y, screen_width(), screen_height());
                }
            }
            next_frame().await;
            continue;
        }

        let mut leave_sjuan = false;
        let mut restart_sjuan = false;
        if let Some(game) = sjuan.as_mut() {
            if game.winner().is_none() && game.current_player() != 0 && get_time() >= ai_next_at {
                let player = game.current_player();
                if let Some(index) = game.legal_indices().first().copied() {
                    game.play(index).map_err(|error| error.to_owned())?;
                    sjuan_status = format!("Bot {} lade ett kort.", player + 1);
                } else {
                    game.pass().map_err(|error| error.to_owned())?;
                    sjuan_status = format!(
                        "Bot {} passar; de andra aktiva spelarna ger ett kort.",
                        player + 1
                    );
                }
                ai_next_at = get_time() + NEXT_TURN_DELAY;
            }
            if game.winner() == Some(0) && !sjuan_rewarded {
                progress.award(20);
                progress::save(&progress)?;
                sjuan_rewarded = true;
                sjuan_status = "Du vann Sjuan! +20 mynt.".to_owned();
            }
            draw_sjuan(
                game,
                sjuan_cursor,
                &sjuan_status,
                screen_width(),
                screen_height(),
                &progress,
            );
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::M) {
                leave_sjuan = true;
                show_menu = true;
            } else if is_key_pressed(KeyCode::N) {
                restart_sjuan = true;
            } else if is_mouse_button_pressed(MouseButton::Left)
                && draw_button_contains(
                    solitaire_menu_button(screen_width(), screen_height()),
                    mouse_position().0,
                    mouse_position().1,
                )
            {
                leave_sjuan = true;
                show_menu = true;
            } else if game.winner().is_none() && game.current_player() == 0 {
                let legal = game.legal_indices();
                if !legal.is_empty() {
                    sjuan_cursor = sjuan_cursor.min(legal.len() - 1);
                    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Up) {
                        sjuan_cursor = sjuan_cursor.saturating_sub(1);
                    }
                    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Down) {
                        sjuan_cursor = (sjuan_cursor + 1).min(legal.len() - 1);
                    }
                    if is_key_pressed(KeyCode::Enter) {
                        let card_index = legal[sjuan_cursor];
                        game.play(card_index).map_err(|error| error.to_owned())?;
                        ai_next_at = get_time() + NEXT_TURN_DELAY;
                    }
                }
                if is_key_pressed(KeyCode::Space) && legal.is_empty() {
                    game.pass().map_err(|error| error.to_owned())?;
                    sjuan_status =
                        "Du passar; de andra aktiva spelarna ger dig ett kort.".to_owned();
                    ai_next_at = get_time() + NEXT_TURN_DELAY;
                }
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (x, y) = mouse_position();
                    if draw_button_contains(
                        sjuan_pass_button(screen_width(), screen_height()),
                        x,
                        y,
                    ) {
                        if legal.is_empty() {
                            game.pass().map_err(|error| error.to_owned())?;
                            sjuan_status =
                                "Du passar; de andra aktiva spelarna ger dig ett kort.".to_owned();
                            ai_next_at = get_time() + NEXT_TURN_DELAY;
                        } else {
                            sjuan_status = "Du måste lägga ett kort om du kan.".to_owned();
                        }
                    } else if let Some(index) = sjuan_card_at(
                        x,
                        y,
                        game.hand(0).unwrap_or(&[]).len(),
                        screen_width(),
                        screen_height(),
                    ) {
                        if let Some(legal_index) = legal.iter().position(|card| *card == index) {
                            sjuan_cursor = legal_index;
                            game.play(index).map_err(|error| error.to_owned())?;
                            ai_next_at = get_time() + NEXT_TURN_DELAY;
                        } else {
                            sjuan_status = "Det kortet kan inte läggas ännu.".to_owned();
                        }
                    }
                }
            }
            if is_key_pressed(KeyCode::Q) {
                break;
            }
            if leave_sjuan {
                sjuan = None;
            } else if restart_sjuan {
                sjuan = Some(sjuan::Sjuan::new(players, seed_or_random(seed)?)?);
                sjuan_rewarded = false;
                sjuan_cursor = 0;
                sjuan_status = "Lägg en sjua först. Bygg sedan vidare i samma färg.".to_owned();
                ai_next_at = 0.0;
            }
            next_frame().await;
            continue;
        }

        if game_session.is_none() {
            draw_setup_screen(players, seed.is_some(), screen_width(), screen_height());
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Down) {
                players = players.saturating_sub(1).max(2);
            }
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Up) {
                players = (players + 1).min(5);
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mouse_x, mouse_y) = mouse_position();
                let (minus, plus) = setup_player_buttons(screen_width(), screen_height());
                if draw_button_contains(minus, mouse_x, mouse_y) {
                    players = players.saturating_sub(1).max(2);
                } else if draw_button_contains(plus, mouse_x, mouse_y) {
                    players = (players + 1).min(5);
                }
            }
            if is_key_pressed(KeyCode::Enter)
                || (is_mouse_button_pressed(MouseButton::Left)
                    && start_button_contains(mouse_position().0, mouse_position().1))
            {
                let game_seed = match seed {
                    Some(seed) => seed,
                    None => random_seed()?,
                };
                game_session = Some(Game::new(players, game_seed)?);
                vandtia_rewarded = false;
            }
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::M) {
                show_menu = true;
            }
            if is_key_pressed(KeyCode::Q) {
                break;
            }
            next_frame().await;
            continue;
        }

        if is_key_pressed(KeyCode::Escape) {
            game_session = None;
            show_menu = true;
            cursor = 0;
            selected.clear();
            status = INITIAL_STATUS.to_owned();
            ai_next_at = 0.0;
            next_frame().await;
            continue;
        }

        let game = game_session.as_mut().expect("game initialized above");
        let current_player = game.current_player();
        let phase = game.player_phase(0).unwrap_or(PlayerPhase::FaceDown);
        let display_order = player_display_order(game, phase);
        if game.winner().is_none() && current_player != 0 && get_time() >= ai_next_at {
            match play_computer_turn(game) {
                Ok(outcome) => {
                    record_successful_turn(
                        outcome,
                        &format!("Bot {} ", current_player + 1),
                        &mut status,
                        &mut cursor,
                        &mut selected,
                        &mut ai_next_at,
                    );
                }
                Err(error) => return Err(error.into()),
            }
        }

        if let Some(winner) = game.winner() {
            if winner == 0 && !vandtia_rewarded {
                progress.award(25);
                progress::save(&progress)?;
                vandtia_rewarded = true;
            }
            draw_game_over_screen(winner, screen_width(), screen_height(), progress.coins);
            let (mouse_x, mouse_y) = mouse_position();
            let return_to_menu = is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::M)
                || is_key_pressed(KeyCode::Escape)
                || (is_mouse_button_pressed(MouseButton::Left)
                    && menu_button_contains(mouse_x, mouse_y));
            if return_to_menu {
                game_session = None;
                vandtia_rewarded = false;
                show_menu = true;
                cursor = 0;
                selected.clear();
                status = INITIAL_STATUS.to_owned();
                ai_next_at = 0.0;
            }
            if is_key_pressed(KeyCode::Q) {
                break;
            }
            next_frame().await;
            continue;
        }

        draw_screen(game, cursor, &selected, &status, &progress);

        if current_player == 0 {
            let count = player_card_count(game, phase);
            if count > 0 {
                cursor = cursor.min(count - 1);
            } else {
                cursor = 0;
            }
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Up) {
                cursor = cursor.saturating_sub(1);
            }
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Down) {
                cursor = cursor.saturating_add(1).min(count.saturating_sub(1));
            }
            if (is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::M))
                && phase != PlayerPhase::FaceDown
                && count > 0
            {
                toggle_selection(&mut selected, display_order[cursor]);
                status = selection_status(&selected);
            }
            if is_key_pressed(KeyCode::D) && phase == PlayerPhase::Hand {
                match game.draw_and_play() {
                    Ok(outcome) => {
                        record_successful_turn(
                            outcome,
                            "Du ",
                            &mut status,
                            &mut cursor,
                            &mut selected,
                            &mut ai_next_at,
                        );
                    }
                    Err(error) => status = format!("Kunde inte dra kort: {error}"),
                }
            } else if is_key_pressed(KeyCode::P)
                && phase != PlayerPhase::FaceDown
                && !game.pile().is_empty()
            {
                match game.pick_up_pile() {
                    Ok(outcome) => {
                        record_successful_turn(
                            outcome,
                            "Du ",
                            &mut status,
                            &mut cursor,
                            &mut selected,
                            &mut ai_next_at,
                        );
                    }
                    Err(error) => status = format!("Kunde inte ta upp spelhögen: {error}"),
                }
            } else if is_key_pressed(KeyCode::Enter) && count > 0 {
                let indices = selected_indices(&selected, cursor, phase);
                if phase == PlayerPhase::FaceDown {
                    match game.play_face_down_at(cursor) {
                        Ok(outcome) => {
                            record_successful_turn(
                                outcome,
                                "Du ",
                                &mut status,
                                &mut cursor,
                                &mut selected,
                                &mut ai_next_at,
                            );
                        }
                        Err(error) => status = format!("Kunde inte vända kortet: {error}"),
                    }
                } else if indices.is_empty() {
                    status = "Markera minst ett kort först.".to_owned();
                } else {
                    play_selected(
                        game,
                        phase,
                        &indices,
                        &mut selected,
                        &mut status,
                        &mut cursor,
                        &mut ai_next_at,
                    );
                }
            }
            if is_key_pressed(KeyCode::Q) {
                break;
            }

            if is_mouse_button_pressed(MouseButton::Left) {
                let (mouse_x, mouse_y) = mouse_position();
                if let Some(slot) =
                    card_at_position(mouse_x, mouse_y, count, screen_width(), screen_height())
                {
                    cursor = slot;
                    let index = display_order[slot];
                    if phase == PlayerPhase::FaceDown {
                        match game.play_face_down_at(index) {
                            Ok(outcome) => {
                                record_successful_turn(
                                    outcome,
                                    "Du ",
                                    &mut status,
                                    &mut cursor,
                                    &mut selected,
                                    &mut ai_next_at,
                                );
                            }
                            Err(error) => status = format!("Kunde inte vända kortet: {error}"),
                        }
                    } else {
                        toggle_selection(&mut selected, index);
                        status = selection_status(&selected);
                    }
                } else if phase != PlayerPhase::FaceDown
                    && play_button_contains(mouse_x, mouse_y, screen_width(), screen_height())
                {
                    if selected.is_empty() {
                        status = "Markera minst ett kort först.".to_owned();
                    } else {
                        let indices = selected.clone();
                        play_selected(
                            game,
                            phase,
                            &indices,
                            &mut selected,
                            &mut status,
                            &mut cursor,
                            &mut ai_next_at,
                        );
                    }
                } else if phase == PlayerPhase::Hand
                    && draw_button_contains_at(mouse_x, mouse_y, screen_width(), screen_height())
                {
                    match game.draw_and_play() {
                        Ok(outcome) => {
                            record_successful_turn(
                                outcome,
                                "Du ",
                                &mut status,
                                &mut cursor,
                                &mut selected,
                                &mut ai_next_at,
                            );
                        }
                        Err(error) => status = format!("Kunde inte dra kort: {error}"),
                    }
                } else if phase != PlayerPhase::FaceDown
                    && !game.pile().is_empty()
                    && pickup_button_contains(mouse_x, mouse_y, screen_width(), screen_height())
                {
                    match game.pick_up_pile() {
                        Ok(outcome) => {
                            record_successful_turn(
                                outcome,
                                "Du ",
                                &mut status,
                                &mut cursor,
                                &mut selected,
                                &mut ai_next_at,
                            );
                        }
                        Err(error) => status = format!("Kunde inte ta upp spelhögen: {error}"),
                    }
                } else if phase != PlayerPhase::FaceDown
                    && pile_target_contains(mouse_x, mouse_y, screen_width(), screen_height())
                {
                    if selected.is_empty() {
                        status = "Markera minst ett kort först.".to_owned();
                    } else {
                        let indices = selected.clone();
                        play_selected(
                            game,
                            phase,
                            &indices,
                            &mut selected,
                            &mut status,
                            &mut cursor,
                            &mut ai_next_at,
                        );
                    }
                }
            }
        } else if is_key_pressed(KeyCode::Q) {
            break;
        }

        next_frame().await;
    }

    Ok(())
}

fn play_selected(
    game: &mut Game,
    phase: PlayerPhase,
    indices: &[usize],
    selection: &mut Vec<usize>,
    status: &mut String,
    cursor: &mut usize,
    ai_next_at: &mut f64,
) {
    let result = match phase {
        PlayerPhase::Hand => game.play_hand_indices(indices),
        PlayerPhase::FaceUp => game.play_face_up_indices(indices),
        PlayerPhase::FaceDown => return,
    };
    match result {
        Ok(outcome) => {
            record_successful_turn(outcome, "Du ", status, cursor, selection, ai_next_at);
        }
        Err(error) => *status = format!("Kunde inte spela de markerade korten: {error}"),
    }
}

fn record_successful_turn(
    outcome: TurnOutcome,
    actor: &str,
    status: &mut String,
    cursor: &mut usize,
    selection: &mut Vec<usize>,
    ai_next_at: &mut f64,
) {
    *status = format_outcome(actor, &outcome);
    *cursor = 0;
    selection.clear();
    *ai_next_at = get_time() + NEXT_TURN_DELAY;
}

fn selection_status(selected: &[usize]) -> String {
    if selected.is_empty() {
        "Markeringen har tagits bort.".to_owned()
    } else {
        format!(
            "{} kort markerade. Klicka på SPELA MARKERADE eller spelhögen.",
            selected.len()
        )
    }
}

fn parse_args() -> Result<(usize, Option<u64>), Box<dyn std::error::Error>> {
    let mut players = 4usize;
    let mut seed = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--players" | "-p" => {
                players = args.next().ok_or("--players kräver ett värde")?.parse()?;
            }
            "--seed" | "-s" => {
                seed = Some(args.next().ok_or("--seed kräver ett värde")?.parse()?);
            }
            "--help" | "-h" => {
                println!("Användning: vandtia-graphics [--players 2-5] [--seed N]");
                println!("Utan --seed skapas ett nytt startvärde när spelet startar.");
                process::exit(0);
            }
            unknown => return Err(format!("okänt argument: {unknown}").into()),
        }
    }
    if !(2..=5).contains(&players) {
        return Err("--players måste vara mellan 2 och 5".into());
    }
    Ok((players, seed))
}

#[cfg(not(target_arch = "wasm32"))]
fn random_seed() -> Result<u64, std::time::SystemTimeError> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64;
    let process_salt = u64::from(process::id()).rotate_left(17);
    let frame_salt = (get_time() * 1_000_000_000.0) as u64;
    Ok(time ^ process_salt ^ frame_salt.rotate_left(31))
}

#[cfg(target_arch = "wasm32")]
fn random_seed() -> Result<u64, std::convert::Infallible> {
    let clock = (get_time() * 1_000_000_000.0) as u64;
    let random = u64::from(macroquad::rand::gen_range(0_u32, u32::MAX));
    Ok(clock ^ random.rotate_left(29) ^ (random << 32))
}

fn seed_or_random(seed: Option<u64>) -> Result<u64, Box<dyn std::error::Error>> {
    match seed {
        Some(seed) => Ok(seed),
        None => Ok(random_seed()?),
    }
}

fn ascii_game_text(text: &str) -> String {
    let mut ascii = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            'å' | 'ä' => ascii.push('a'),
            'ö' => ascii.push('o'),
            'Å' | 'Ä' => ascii.push('A'),
            'Ö' => ascii.push('O'),
            'é' | 'è' | 'ê' => ascii.push('e'),
            'É' | 'È' | 'Ê' => ascii.push('E'),
            '•' => ascii.push('*'),
            '–' | '—' => ascii.push('-'),
            '→' | '➜' => ascii.push_str("->"),
            '←' => ascii.push_str("<-"),
            '↑' => ascii.push('^'),
            '↓' => ascii.push('v'),
            '×' => ascii.push('x'),
            '…' => ascii.push_str("..."),
            character if character.is_ascii() => ascii.push(character),
            _ => ascii.push('?'),
        }
    }
    ascii
}

fn draw_text(
    text: impl AsRef<str>,
    x: f32,
    y: f32,
    font_size: f32,
    color: Color,
) -> TextDimensions {
    macroquad::prelude::draw_text(&ascii_game_text(text.as_ref()), x, y, font_size, color)
}

fn measure_text(
    text: impl AsRef<str>,
    font: Option<&Font>,
    font_size: u16,
    font_scale: f32,
) -> TextDimensions {
    macroquad::prelude::measure_text(&ascii_game_text(text.as_ref()), font, font_size, font_scale)
}

fn draw_main_menu(width: f32, height: f32, progress: &progress::Progress) {
    clear_background(TABLE_COLOR);
    let compact = compact_layout(width, height);
    let title_y = if height < 520.0 {
        height * 0.16
    } else {
        height * 0.18
    };
    draw_centered_text(
        "KORTSPEL",
        width / 2.0,
        title_y,
        if compact { 38 } else { 54 },
        GOLD,
    );
    draw_centered_text(
        "Välj ett spel",
        width / 2.0,
        title_y + if compact { 32.0 } else { 42.0 },
        if compact { 20 } else { 24 },
        WHITE,
    );
    let labels = ["VÄNDTIA", "SJUAN", "PATIENS", "KORTBUTIK", "KORTGALLERI"];
    for (rect, label) in main_menu_buttons(width, height).into_iter().zip(labels) {
        draw_button(rect, label, true);
    }
    draw_centered_text(
        &format!("{} mynt  •  1–5: välj  •  Q: avsluta", progress.coins),
        width / 2.0,
        height - 8.0,
        if compact { 12 } else { 16 },
        LIGHTGRAY,
    );
}

fn main_menu_buttons(width: f32, height: f32) -> [Rect; 5] {
    let button_width = (width - 24.0).clamp(160.0, 340.0).min(width - 12.0);
    let button_height = ((height - 240.0) / 5.0).clamp(30.0, 58.0);
    let gap = 8.0;
    let total_height = 5.0 * button_height + 4.0 * gap;
    let title_y = if height < 520.0 {
        height * 0.16
    } else {
        height * 0.18
    };
    let first_y = ((height - total_height) / 2.0 + 45.0).max(title_y + 72.0);
    std::array::from_fn(|index| {
        Rect::new(
            (width - button_width) / 2.0,
            first_y + index as f32 * (button_height + gap),
            button_width,
            button_height,
        )
    })
}

fn vandtia_menu_button(width: f32, height: f32) -> Rect {
    main_menu_buttons(width, height)[0]
}

fn sjuan_menu_option_rect(width: f32, height: f32) -> Rect {
    main_menu_buttons(width, height)[1]
}

fn solitaire_menu_button(width: f32, _height: f32) -> Rect {
    let button_width = (width - 16.0).clamp(72.0, 104.0);
    Rect::new(
        (width - button_width - 8.0).max(0.0),
        8.0,
        button_width,
        36.0,
    )
}

fn solitaire_menu_option_rect(width: f32, height: f32) -> Rect {
    main_menu_buttons(width, height)[2]
}

fn shop_menu_option_rect(width: f32, height: f32) -> Rect {
    main_menu_buttons(width, height)[3]
}

fn card_gallery_menu_option_rect(width: f32, height: f32) -> Rect {
    main_menu_buttons(width, height)[4]
}

fn card_gallery_menu_button(width: f32, _height: f32) -> Rect {
    let button_width = (width - 16.0).clamp(72.0, 104.0);
    Rect::new(
        (width - button_width - 8.0).max(0.0),
        8.0,
        button_width,
        36.0,
    )
}

fn card_gallery_size_buttons(width: f32, height: f32) -> [Rect; 3] {
    let button_width = (width * 0.22).clamp(72.0, 120.0);
    let gap = 10.0;
    let total_width = button_width * 3.0 + gap * 2.0;
    let start_x = (width - total_width) / 2.0;
    let y = if height < 520.0 { 44.0 } else { 62.0 };
    std::array::from_fn(|index| {
        Rect::new(
            start_x + index as f32 * (button_width + gap),
            y,
            button_width,
            if compact_layout(width, height) {
                42.0
            } else {
                46.0
            },
        )
    })
}

fn card_gallery_layout(width: f32, height: f32, size: usize) -> (f32, f32, f32) {
    let scale = [0.45, 0.7, 1.0][size.min(2)];
    let available_width = (width - 28.0) / 13.0;
    let selector = card_gallery_size_buttons(width, height);
    let grid_start = selector[0].y + selector[0].h + 18.0;
    let available_height = (height - grid_start - 50.0) / 4.0;
    let card_width = (CARD_WIDTH * scale)
        .min(available_width)
        .min(available_height * CARD_WIDTH / CARD_HEIGHT)
        .max(1.0);
    let card_height = card_width * CARD_HEIGHT / CARD_WIDTH;
    let gap = ((width - 28.0 - card_width * 13.0) / 12.0).max(0.0);
    (card_width, card_height, gap.max(0.0))
}

fn draw_card_gallery(width: f32, height: f32, selected_size: usize, progress: &progress::Progress) {
    clear_background(TABLE_COLOR);
    draw_centered_text(
        "KORTGALLERI",
        width / 2.0,
        if height < 520.0 { 28.0 } else { 42.0 },
        if compact_layout(width, height) {
            22
        } else {
            30
        },
        GOLD,
    );
    let labels = ["LITEN 1", "NORMAL 2", "STOR 3"];
    for (index, rect) in card_gallery_size_buttons(width, height)
        .into_iter()
        .enumerate()
    {
        let label = if index == selected_size {
            format!("[ {} ]", labels[index])
        } else {
            labels[index].to_owned()
        };
        draw_setup_button(rect, &label);
    }

    let (card_width, card_height, gap) = card_gallery_layout(width, height, selected_size);
    let row_gap = if height < 520.0 { 9.0 } else { 18.0 };
    let selector = card_gallery_size_buttons(width, height);
    let start_y = selector[0].y + selector[0].h + 18.0;
    let deck = vandtia::cards::standard_deck();
    for (index, card) in deck.into_iter().enumerate() {
        let column = index % 13;
        let row = index / 13;
        let x = 14.0 + column as f32 * (card_width + gap);
        let y = start_y + row as f32 * (card_height + row_gap);
        card_renderer::draw_face_with_theme(
            Rect::new(x, y, card_width, card_height),
            card,
            CardStyle::default(),
            progress.front_theme,
        );
    }

    let menu = card_gallery_menu_button(width, height);
    draw_button(menu, "MENY", true);
    draw_centered_text(
        "52 kort • Kortbutiken ändrar utseendet • M: meny",
        width / 2.0,
        height - 8.0,
        13,
        LIGHTGRAY,
    );
}

fn shop_tab_buttons(width: f32) -> [Rect; 2] {
    let button_width = ((width - 100.0) / 2.0).clamp(72.0, 180.0);
    let gap = 8.0;
    let start_x = (width - button_width * 2.0 - gap) / 2.0;
    [
        Rect::new(start_x, 68.0, button_width, 32.0),
        Rect::new(start_x + button_width + gap, 68.0, button_width, 32.0),
    ]
}

fn shop_tab_at(x: f32, y: f32, width: f32) -> Option<bool> {
    shop_tab_buttons(width)
        .into_iter()
        .enumerate()
        .find_map(|(index, rect)| draw_button_contains(rect, x, y).then_some(index == 0))
}

fn shop_choice_rects(width: f32, height: f32) -> [Rect; 6] {
    let margin = 10.0;
    let gap_x = 6.0;
    let gap_y = 8.0;
    let item_width = (width - margin * 2.0 - gap_x * 2.0) / 3.0;
    let item_height = ((height - 205.0) / 2.0).clamp(56.0, 112.0);
    let start_y = 108.0;
    std::array::from_fn(|index| {
        let row = index / 3;
        let column = index % 3;
        Rect::new(
            margin + column as f32 * (item_width + gap_x),
            start_y + row as f32 * (item_height + gap_y),
            item_width,
            item_height,
        )
    })
}

fn shop_choice_at(x: f32, y: f32, width: f32, height: f32) -> Option<usize> {
    shop_choice_rects(width, height)
        .into_iter()
        .enumerate()
        .find_map(|(index, rect)| draw_button_contains(rect, x, y).then_some(index))
}

fn draw_shop(
    width: f32,
    height: f32,
    progress: &progress::Progress,
    front_tab: bool,
    status: &str,
) {
    clear_background(TABLE_COLOR);
    draw_centered_text(
        "KORTBUTIK",
        width / 2.0,
        36.0,
        if compact_layout(width, height) {
            24
        } else {
            30
        },
        GOLD,
    );
    draw_centered_text(
        &format!("Plånbok: {} mynt", progress.coins),
        width / 2.0,
        64.0,
        18,
        YELLOW,
    );
    for (index, rect) in shop_tab_buttons(width).into_iter().enumerate() {
        draw_button(
            rect,
            if index == 0 { "FRAMSIDA" } else { "BAKSIDA" },
            (index == 0) == front_tab,
        );
    }

    let number_sample = Card {
        rank: Rank::Seven,
        suit: Suit::Hearts,
    };
    let face_sample = Card {
        rank: Rank::Queen,
        suit: Suit::Diamonds,
    };
    let names = if front_tab {
        [
            "KLASSISK",
            "SKOGSSAGA",
            "SOLNEDGÅNG",
            "CYBERPUNK",
            "KUNGLIG",
            "SERIEFIGUR",
        ]
    } else {
        [
            "MOSAIC",
            "LÖVVERK",
            "SOLSTRÅLAR",
            "KRETSKORT",
            "KOMPASS",
            "KARAMELL",
        ]
    };
    let prices = if front_tab {
        &progress::FRONT_PRICES
    } else {
        &progress::BACK_PRICES
    };
    for (theme, rect) in shop_choice_rects(width, height).into_iter().enumerate() {
        let owned = if front_tab {
            progress.owns_front(theme)
        } else {
            progress.owns_back(theme)
        };
        let selected = if front_tab {
            progress.front_theme == theme
        } else {
            progress.back_theme == theme
        };
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.02, 0.14, 0.1, 1.0),
        );
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            2.0,
            if selected { GOLD } else { DARKGREEN },
        );
        let preview_width = (rect.w * 0.38)
            .min(rect.h * CARD_WIDTH / CARD_HEIGHT * 0.72)
            .max(18.0);
        let preview_height = preview_width * CARD_HEIGHT / CARD_WIDTH;
        let preview_x = rect.x + 5.0;
        let preview_y = rect.y + (rect.h - preview_height) / 2.0;
        if front_tab {
            card_renderer::draw_face_with_theme(
                Rect::new(preview_x, preview_y, preview_width, preview_height),
                number_sample,
                CardStyle::default(),
                theme,
            );
            let face_width = preview_width * 0.78;
            card_renderer::draw_face_with_theme(
                Rect::new(
                    preview_x + preview_width * 0.62,
                    preview_y + preview_height * 0.12,
                    face_width,
                    preview_height * 0.78,
                ),
                face_sample,
                CardStyle::default(),
                theme,
            );
        } else {
            card_renderer::draw_back_with_theme(
                Rect::new(preview_x, preview_y, preview_width, preview_height),
                false,
                theme,
            );
        }
        let label_x = rect.x + rect.w * 0.48;
        let label_size = if width < 400.0 { 8.0 } else { 11.0 };
        draw_text(
            names[theme],
            label_x,
            rect.y + rect.h * 0.43,
            label_size,
            WHITE,
        );
        let action = if selected {
            "VALD".to_owned()
        } else if owned {
            "VÄLJ".to_owned()
        } else {
            format!("{} MYNT", prices[theme])
        };
        draw_text(
            &action,
            label_x,
            rect.y + rect.h * 0.7,
            label_size,
            if owned { YELLOW } else { LIGHTGRAY },
        );
    }

    let menu = card_gallery_menu_button(width, height);
    draw_button(menu, "MENY", true);
    draw_centered_text(
        status,
        width / 2.0,
        height - 20.0,
        13,
        if status.contains("inte") {
            YELLOW
        } else {
            LIGHTGRAY
        },
    );
    draw_centered_text(
        "Sex unika stilar • Köp eller välj • M: meny",
        width / 2.0,
        height - 4.0,
        10,
        LIGHTGRAY,
    );
}

fn sjuan_board_layout(width: f32, height: f32) -> (f32, f32, f32, f32) {
    let card_width = ((width - 54.0) / 13.0)
        .min(((height - 270.0).max(80.0) / 4.0) * CARD_WIDTH / CARD_HEIGHT)
        .clamp(14.0, 70.0);
    let card_height = card_width * CARD_HEIGHT / CARD_WIDTH;
    let start_x = 40.0 + (width - 48.0 - card_width * 13.0).max(0.0) / 2.0;
    (card_width, card_height, start_x, 72.0)
}

fn sjuan_hand_layout(count: usize, width: f32, height: f32) -> (f32, f32, f32) {
    let card_height = (height * 0.16).clamp(60.0, 110.0);
    let natural_width = card_height * CARD_WIDTH / CARD_HEIGHT;
    let card_width = natural_width.min((width - 20.0).max(1.0));
    let gap = if count > 1 {
        ((width - 20.0 - card_width) / (count - 1) as f32 - card_width).min(8.0)
    } else {
        0.0
    };
    let total_width = if count == 0 {
        0.0
    } else {
        card_width * count as f32 + gap * count.saturating_sub(1) as f32
    };
    let start_x = ((width - total_width) / 2.0).max(10.0);
    (card_width, gap, start_x)
}

fn sjuan_hand_y(height: f32, card_width: f32) -> f32 {
    height - card_width * CARD_HEIGHT / CARD_WIDTH - 100.0
}

fn sjuan_card_at(x: f32, y: f32, count: usize, width: f32, height: f32) -> Option<usize> {
    let (card_width, gap, start_x) = sjuan_hand_layout(count, width, height);
    let card_y = sjuan_hand_y(height, card_width);
    (0..count).rev().find(|index| {
        let card_x = start_x + *index as f32 * (card_width + gap);
        x >= card_x
            && x <= card_x + card_width
            && y >= card_y
            && y <= card_y + card_width * CARD_HEIGHT / CARD_WIDTH
    })
}

fn sjuan_pass_button(width: f32, height: f32) -> Rect {
    Rect::new((width - 130.0) / 2.0, height - 62.0, 130.0, 48.0)
}

fn draw_sjuan(
    game: &sjuan::Sjuan,
    cursor: usize,
    status: &str,
    width: f32,
    height: f32,
    progress: &progress::Progress,
) {
    clear_background(TABLE_COLOR);
    draw_text(
        "SJUAN",
        12.0,
        27.0,
        if compact_layout(width, height) {
            20.0
        } else {
            26.0
        },
        GOLD,
    );
    draw_text(
        &format!("{} mynt", progress.coins),
        100.0,
        26.0,
        16.0,
        YELLOW,
    );
    let menu = solitaire_menu_button(width, height);
    draw_button(menu, "MENY", true);

    let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
    let ranks = [
        Rank::Ace,
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
    ];
    let (card_width, card_height, start_x, start_y) = sjuan_board_layout(width, height);
    let row_gap = (card_height * 0.12).max(3.0);
    for (row, suit) in suits.into_iter().enumerate() {
        let y = start_y + row as f32 * (card_height + row_gap);
        draw_text(
            match suit {
                Suit::Clubs => "K",
                Suit::Diamonds => "R",
                Suit::Hearts => "H",
                Suit::Spades => "S",
            },
            12.0,
            y + card_height * 0.66,
            (card_height * 0.5).clamp(12.0, 24.0),
            match suit {
                Suit::Diamonds | Suit::Hearts => RED,
                _ => WHITE,
            },
        );
        for (column, rank) in ranks.iter().copied().enumerate() {
            let rect = Rect::new(
                start_x + column as f32 * card_width,
                y,
                card_width - 2.0,
                card_height,
            );
            if game.has_card(suit, rank) {
                card_renderer::draw_face_with_theme(
                    rect,
                    Card { rank, suit },
                    CardStyle::default(),
                    progress.front_theme,
                );
            } else {
                draw_slot(rect);
            }
        }
    }

    let hand = game.hand(0).unwrap_or(&[]);
    let legal = if game.current_player() == 0 {
        game.legal_indices()
    } else {
        Vec::new()
    };
    let (hand_card_width, hand_gap, hand_start_x) = sjuan_hand_layout(hand.len(), width, height);
    let hand_y = sjuan_hand_y(height, hand_card_width);
    for (index, card) in hand.iter().copied().enumerate() {
        let is_legal = legal.contains(&index);
        let focused = is_legal && legal.get(cursor) == Some(&index);
        card_renderer::draw_face_with_theme(
            Rect::new(
                hand_start_x + index as f32 * (hand_card_width + hand_gap),
                hand_y,
                hand_card_width,
                hand_card_width * CARD_HEIGHT / CARD_WIDTH,
            ),
            card,
            CardStyle {
                selected: is_legal,
                focused,
            },
            progress.front_theme,
        );
    }

    let turn = if game.current_player() == 0 {
        "Din tur".to_owned()
    } else {
        format!("Bot {}:s tur", game.current_player() + 1)
    };
    draw_centered_text(
        &format!("{turn}  •  {} aktiva", game.active_player_count()),
        width / 2.0,
        49.0,
        15,
        SKYBLUE,
    );
    let status_width = measure_text(status, None, 14, 1.0).width;
    let status_scale = ((width - 18.0) / status_width.max(1.0)).min(1.0);
    draw_text(
        status,
        (width - status_width * status_scale) / 2.0,
        hand_y - 10.0,
        14.0 * status_scale,
        YELLOW,
    );
    let can_pass = game.winner().is_none() && legal.is_empty() && game.current_player() == 0;
    draw_button(
        sjuan_pass_button(width, height),
        if can_pass { "PASSA" } else { "PASSA?" },
        can_pass,
    );
    draw_centered_text(
        "Lägg sjuor och bygg utåt i samma färg • Piltangenter/Retur • M: meny",
        width / 2.0,
        height - 3.0,
        10,
        LIGHTGRAY,
    );
    if let Some(winner) = game.winner() {
        let message = if winner == 0 {
            "DU VANN SJUAN! +20 MYNT"
        } else {
            "SPELET ÄR SLUT"
        };
        draw_centered_text(message, width / 2.0, height / 2.0, 30, GOLD);
        draw_centered_text(
            "N: NYTT SPEL   M: MENY",
            width / 2.0,
            height / 2.0 + 34.0,
            18,
            WHITE,
        );
    }
}

fn solitaire_board_layout(width: f32, height: f32) -> SolitaireBoardLayout {
    let card_width = ((width - 28.0) / 7.4)
        .min((height - 240.0).max(68.0) * CARD_WIDTH / (2.0 * CARD_HEIGHT))
        .clamp(34.0, 96.0);
    let card_height = card_width * CARD_HEIGHT / CARD_WIDTH;
    let margin = ((width - 7.0 * card_width) / 8.0).max(2.0);
    let columns = std::array::from_fn(|column| margin + column as f32 * (card_width + margin));
    let top_y = 50.0;
    let tableau_y = top_y + card_height + 18.0;
    SolitaireBoardLayout {
        card_width,
        card_height,
        columns,
        stock: Rect::new(columns[0], top_y, card_width, card_height),
        waste: Rect::new(columns[1], top_y, card_width, card_height),
        foundations: std::array::from_fn(|index| {
            Rect::new(columns[index + 3], top_y, card_width, card_height)
        }),
        tableau_y,
        face_down_step: (card_height * 0.22).clamp(10.0, 20.0),
        face_up_step: (card_height * 0.32).clamp(14.0, 30.0),
    }
}

struct SolitaireBoardLayout {
    card_width: f32,
    card_height: f32,
    columns: [f32; 7],
    stock: Rect,
    waste: Rect,
    foundations: [Rect; 4],
    tableau_y: f32,
    face_down_step: f32,
    face_up_step: f32,
}

fn draw_solitaire(
    game: &solitaire::Solitaire,
    width: f32,
    height: f32,
    progress: &progress::Progress,
) {
    clear_background(TABLE_COLOR);
    let layout = solitaire_board_layout(width, height);
    draw_text(
        "PATIENS",
        12.0,
        27.0,
        if width < 500.0 { 18.0 } else { 24.0 },
        GOLD,
    );
    draw_text(
        &format!("{} mynt", progress.coins),
        if width < 500.0 { 92.0 } else { 150.0 },
        26.0,
        if width < 500.0 { 13.0 } else { 16.0 },
        YELLOW,
    );
    let menu_button = solitaire_menu_button(width, height);
    draw_button(menu_button, "MENY", true);
    draw_slot(layout.stock);
    if game.stock_count() > 0 {
        card_renderer::draw_back_with_theme(layout.stock, false, progress.back_theme);
    }
    draw_slot(layout.waste);
    let waste_visible = game.waste_visible();
    let waste_offset = layout.card_width * 0.18;
    for (index, card) in waste_visible.iter().enumerate() {
        let is_top = index + 1 == waste_visible.len();
        card_renderer::draw_face_with_theme(
            Rect::new(
                layout.waste.x + index as f32 * waste_offset,
                layout.waste.y,
                layout.card_width,
                layout.card_height,
            ),
            *card,
            CardStyle {
                selected: is_top && game.is_waste_selected(),
                focused: false,
            },
            progress.front_theme,
        );
    }
    for (index, rect) in layout.foundations.iter().enumerate() {
        draw_slot(*rect);
        if let Some(card) = game.foundation_top(index) {
            card_renderer::draw_face_with_theme(
                *rect,
                card,
                CardStyle {
                    selected: game.is_foundation_selected(index),
                    focused: false,
                },
                progress.front_theme,
            );
        }
    }
    for (column, cards) in game.tableau().iter().enumerate() {
        let mut y = layout.tableau_y;
        if cards.is_empty() {
            draw_slot(Rect::new(
                layout.columns[column],
                y,
                layout.card_width,
                layout.card_height,
            ));
        }
        for (index, card) in cards.iter().enumerate() {
            let rect = Rect::new(
                layout.columns[column],
                y,
                layout.card_width,
                layout.card_height,
            );
            if card.face_up {
                card_renderer::draw_face_with_theme(
                    rect,
                    card.card,
                    CardStyle {
                        selected: game.is_tableau_selected(column, index),
                        focused: false,
                    },
                    progress.front_theme,
                );
            } else {
                card_renderer::draw_back_with_theme(rect, false, progress.back_theme);
            }
            y += if card.face_up {
                layout.face_up_step
            } else {
                layout.face_down_step
            };
        }
    }
    let status_y = height - 34.0;
    draw_rectangle(0.0, status_y - 20.0, width, 54.0, PANEL_COLOR);
    let status_size = if width < 500.0 { 12 } else { 17 };
    let status = if game.no_moves_before_stock_recycle() {
        "Inga drag kvar denna runda. Klicka på talongen för att blanda om."
    } else {
        game.status()
    };
    let status_width = measure_text(status, None, status_size, 1.0).width;
    let scale = ((width - 16.0) / status_width.max(1.0)).min(1.0);
    draw_text(
        status,
        (width - status_width * scale) / 2.0,
        status_y,
        f32::from(status_size) * scale,
        YELLOW,
    );
    draw_centered_text(
        "Talong: dra 3 kort   Spela översta öppna kortet   M: meny   N: nytt spel",
        width / 2.0,
        height - 7.0,
        11,
        LIGHTGRAY,
    );
    if game.has_won() {
        draw_centered_text("DU VANN! +15 MYNT", width / 2.0, height / 2.0, 34, GOLD);
        draw_centered_text(
            "N: NYTT SPEL   M: MENY",
            width / 2.0,
            height / 2.0 + 34.0,
            18,
            WHITE,
        );
    }
}

fn draw_slot(rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.02, 0.18, 0.12, 1.0),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, DARKGREEN);
}

fn handle_solitaire_click(
    game: &mut solitaire::Solitaire,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let layout = solitaire_board_layout(width, height);
    if draw_button_contains(layout.stock, x, y) {
        game.click_stock();
        return;
    }
    let waste_hit_area = Rect::new(
        layout.waste.x,
        layout.waste.y,
        layout.card_width + layout.card_width * 0.18 * 2.0,
        layout.card_height,
    );
    if draw_button_contains(waste_hit_area, x, y) {
        game.click_waste();
        return;
    }
    for (index, rect) in layout.foundations.iter().enumerate() {
        if draw_button_contains(*rect, x, y) {
            game.click_foundation(index);
            return;
        }
    }

    for column in 0..7 {
        if x < layout.columns[column] || x > layout.columns[column] + layout.card_width {
            continue;
        }
        let cards = &game.tableau()[column];
        if cards.is_empty() {
            if y >= layout.tableau_y && y <= layout.tableau_y + layout.card_height {
                game.click_empty_tableau(column);
                return;
            }
            continue;
        }
        let mut card_y = layout.tableau_y;
        for (index, card) in cards.iter().enumerate() {
            let step = if card.face_up {
                layout.face_up_step
            } else {
                layout.face_down_step
            };
            let visible_end = if index + 1 == cards.len() {
                card_y + layout.card_height
            } else {
                card_y + step
            };
            if y >= card_y && y < visible_end {
                game.click_tableau(column, index);
                return;
            }
            card_y += step;
        }
    }
}

fn draw_setup_screen(players: usize, fixed_seed: bool, width: f32, height: f32) {
    let compact = compact_layout(width, height);
    let short = height < 520.0;
    clear_background(TABLE_COLOR);
    draw_centered_text(
        "VÄNDTIA",
        width / 2.0,
        height * if short { 0.16 } else { 0.22 },
        if compact { 36 } else { 48 },
        GOLD,
    );
    draw_centered_text(
        "Välj antal spelare",
        width / 2.0,
        height * if short { 0.28 } else { 0.36 },
        if compact { 20 } else { 24 },
        WHITE,
    );

    let center_y = setup_player_selector_y(height);
    let (minus, plus) = setup_player_buttons(width, height);
    draw_setup_button(minus, "-");
    draw_setup_button(plus, "+");
    draw_text(
        players.to_string(),
        width / 2.0 - 14.0,
        center_y + 17.0,
        42.0,
        WHITE,
    );
    if !compact {
        draw_centered_text(
            "Använd piltangenterna eller knapparna (2-5 spelare)",
            width / 2.0,
            center_y + 61.0,
            18,
            LIGHTGRAY,
        );
    }
    draw_centered_text(
        if fixed_seed {
            if compact {
                "Fast startvärde"
            } else {
                "Använder startvärdet från kommandoraden"
            }
        } else if compact {
            "Slumpat startvärde"
        } else {
            "Ett slumpat startvärde skapas för spelet"
        },
        width / 2.0,
        center_y + if short { 50.0 } else { 96.0 },
        if compact { 15 } else { 17 },
        LIGHTGRAY,
    );

    let start = setup_start_button(width, height);
    draw_setup_button(start, "STARTA SPELET");
    if !compact {
        draw_centered_text(
            "RETUR: STARTA    Q: AVSLUTA",
            width / 2.0,
            start.y + start.h + 33.0,
            17,
            LIGHTGRAY,
        );
    }
}

fn setup_player_buttons(width: f32, height: f32) -> (Rect, Rect) {
    let center_y = setup_player_selector_y(height);
    let size = if compact_layout(width, height) {
        66.0
    } else {
        58.0
    };
    let distance = 117.0_f32.min((width - size - 32.0).max(0.0) / 2.0);
    (
        Rect::new(
            width / 2.0 - distance - size / 2.0,
            center_y - size / 2.0,
            size,
            size,
        ),
        Rect::new(
            width / 2.0 + distance - size / 2.0,
            center_y - size / 2.0,
            size,
            size,
        ),
    )
}

fn setup_player_selector_y(height: f32) -> f32 {
    height * if height < 520.0 { 0.44 } else { 0.51 }
}

fn setup_start_button(width: f32, height: f32) -> Rect {
    let button_width = (width - 40.0).clamp(210.0, 300.0);
    let button_height = if compact_layout(width, height) {
        64.0
    } else {
        52.0
    };
    Rect::new(
        (width - button_width) / 2.0,
        height * 0.72,
        button_width,
        button_height,
    )
}

fn start_button_contains(x: f32, y: f32) -> bool {
    let start = setup_start_button(screen_width(), screen_height());
    draw_button_contains(start, x, y)
}

fn menu_button_rect(width: f32, height: f32) -> Rect {
    let button_width = (width - 32.0).clamp(170.0, 320.0);
    let button_height = if compact_layout(width, height) {
        64.0
    } else {
        54.0
    };
    Rect::new(
        (width - button_width) / 2.0,
        height * 0.62,
        button_width,
        button_height,
    )
}

fn menu_button_contains(x: f32, y: f32) -> bool {
    draw_button_contains(menu_button_rect(screen_width(), screen_height()), x, y)
}

fn draw_game_over_screen(winner: usize, width: f32, height: f32, coins: u32) {
    let compact = compact_layout(width, height);
    clear_background(TABLE_COLOR);
    let title = if winner == 0 {
        "DU VANN!"
    } else {
        "SPELET ÄR SLUT"
    };
    draw_centered_text(
        title,
        width / 2.0,
        height * 0.28,
        if compact { 34 } else { 42 },
        GOLD,
    );
    let message = if winner == 0 {
        "Du kom först ut!"
    } else {
        "Du blev sist kvar."
    };
    draw_centered_text(
        message,
        width / 2.0,
        height * 0.43,
        if compact { 24 } else { 30 },
        WHITE,
    );
    if winner > 0 {
        let bot = format!("Bot {} kom först ut", winner + 1);
        draw_centered_text(&bot, width / 2.0, height * 0.50, 20, LIGHTGRAY);
    } else {
        draw_centered_text("Du fick 25 mynt!", width / 2.0, height * 0.50, 20, YELLOW);
    }
    draw_centered_text(
        &format!("Plånbok: {coins} mynt"),
        width / 2.0,
        height * 0.56,
        20,
        YELLOW,
    );

    let menu_button = menu_button_rect(width, height);
    draw_setup_button(menu_button, "TILLBAKA TILL MENYN");
    if !compact {
        draw_centered_text(
            "RETUR / M: MENY    Q: AVSLUTA",
            width / 2.0,
            menu_button.y + menu_button.h + 34.0,
            17,
            LIGHTGRAY,
        );
    }
}

fn draw_centered_text(text: &str, center_x: f32, y: f32, font_size: u16, color: Color) {
    let text_width = measure_text(text, None, font_size, 1.0).width;
    draw_text(
        text,
        center_x - text_width / 2.0,
        y,
        f32::from(font_size),
        color,
    );
}

fn draw_setup_button(rect: Rect, label: &str) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.16, 0.48, 0.32, 1.0),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, GOLD);
    let font_size = if rect.h >= 64.0 { 22 } else { 24 };
    let text_width = measure_text(label, None, font_size, 1.0).width;
    draw_text(
        label,
        rect.x + (rect.w - text_width) / 2.0,
        rect.y + rect.h / 2.0 + f32::from(font_size) * 0.35,
        f32::from(font_size),
        WHITE,
    );
}

fn draw_screen(
    game: &Game,
    cursor: usize,
    selected: &[usize],
    status: &str,
    progress: &progress::Progress,
) {
    clear_background(TABLE_COLOR);
    let width = screen_width();
    let height = screen_height();
    let compact = compact_layout(width, height);

    if compact {
        draw_text("VÄNDTIA", 12.0, 28.0, 20.0, GOLD);
    } else {
        draw_text("VÄNDTIA", 28.0, 42.0, 34.0, GOLD);
        draw_text(
            format!(
                "DRAGHÖG: {} kort     SPELHÖG: {} kort",
                game.draw_pile_size(),
                game.pile().len()
            ),
            240.0,
            38.0,
            22.0,
            WHITE,
        );
        draw_text(
            "FÄRGER: C=KLÖVER  D=RUTER  H=HJÄRTER  S=SPADER",
            28.0,
            61.0,
            15.0,
            LIGHTGRAY,
        );
    }
    let current = if game.current_player() == 0 {
        "Din tur".to_owned()
    } else {
        format!("Bot {}:s tur", game.current_player() + 1)
    };
    draw_text(
        &current,
        if compact {
            width - 110.0
        } else {
            width - 210.0
        },
        if compact { 28.0 } else { 38.0 },
        if compact { 16.0 } else { 22.0 },
        SKYBLUE,
    );

    draw_player_tables(game, width, height, progress);
    draw_pile(game, width, height, compact, progress);
    draw_player_cards(game, cursor, selected, width, height, progress);
    draw_controls(game, status, width, height, compact);
}

fn draw_player_tables(game: &Game, width: f32, height: f32, progress: &progress::Progress) {
    if compact_layout(width, height) {
        draw_compact_player_tables(game, width, progress);
        return;
    }

    let columns = opponent_columns(game.player_count(), width);
    for player in 0..game.player_count() {
        let x = 28.0 + (player % columns) as f32 * 234.0;
        let y = 76.0 + (player / columns) as f32 * 154.0;
        draw_rectangle(x, y, 220.0, 140.0, PANEL_COLOR);
        draw_rectangle_lines(x, y, 220.0, 140.0, 2.0, DARKGREEN);
        let label = if player == 0 {
            "YOU".to_owned()
        } else {
            format!("BOT {}", player + 1)
        };
        let hand_count = game.hand_count(player).unwrap_or(0);
        draw_text(
            format!("{label}: {hand_count} PÅ HANDEN"),
            x + 12.0,
            y + 27.0,
            16.0,
            WHITE,
        );
        for (slot, (has_face_down, face_up_card)) in table_stack_layers(
            game.face_up(player).unwrap_or(&[]),
            game.face_down_count(player).unwrap_or(0),
        )
        .into_iter()
        .enumerate()
        {
            let card_x = x + 12.0 + slot as f32 * 58.0;
            let card_y = y + 48.0;
            if has_face_down {
                card_renderer::draw_back_with_theme(
                    Rect::new(card_x, card_y + 5.0, 54.0, 78.0),
                    false,
                    progress.back_theme,
                );
            }
            if let Some(card) = face_up_card {
                card_renderer::draw_face_with_theme(
                    Rect::new(card_x, card_y, 54.0, 78.0),
                    card,
                    CardStyle::default(),
                    progress.front_theme,
                );
            }
        }
    }
}

fn draw_compact_player_tables(game: &Game, width: f32, progress: &progress::Progress) {
    let columns = ((width / 170.0).floor() as usize)
        .max(1)
        .min(game.player_count());
    let gap = 6.0;
    let cell_width = (width - 16.0 - gap * (columns - 1) as f32) / columns as f32;
    let cell_height = 42.0;

    for player in 0..game.player_count() {
        let column = player % columns;
        let row = player / columns;
        let x = 8.0 + column as f32 * (cell_width + gap);
        let y = 42.0 + row as f32 * (cell_height + gap);
        let border = if player == game.current_player() {
            GOLD
        } else {
            DARKGREEN
        };
        draw_rectangle(x, y, cell_width, cell_height, PANEL_COLOR);
        draw_rectangle_lines(x, y, cell_width, cell_height, 2.0, border);

        let label = if player == 0 {
            "DU".to_owned()
        } else {
            format!("BOT {}", player + 1)
        };
        draw_text(
            format!("{label}  {}", game.hand_count(player).unwrap_or(0)),
            x + 7.0,
            y + 16.0,
            12.0,
            WHITE,
        );

        for (index, (has_face_down, face_up_card)) in table_stack_layers(
            game.face_up(player).unwrap_or(&[]),
            game.face_down_count(player).unwrap_or(0),
        )
        .into_iter()
        .enumerate()
        {
            let card_rect = Rect::new(
                x + cell_width - 32.0 * (index + 1) as f32,
                y + 4.0,
                27.0,
                34.0,
            );
            if has_face_down {
                card_renderer::draw_back_with_theme(card_rect, false, progress.back_theme);
            }
            if let Some(card) = face_up_card {
                card_renderer::draw_face_with_theme(
                    card_rect,
                    card,
                    CardStyle::default(),
                    progress.front_theme,
                );
            }
        }
    }
}

fn opponent_columns(player_count: usize, width: f32) -> usize {
    let available_width = (width - 56.0).max(220.0);
    let columns = ((available_width + 14.0) / 234.0).floor() as usize;
    columns.max(1).min(player_count.max(1))
}

fn table_stack_layers(face_up: &[Card], face_down_count: usize) -> [(bool, Option<Card>); 3] {
    std::array::from_fn(|index| (index < face_down_count, face_up.get(index).copied()))
}

fn draw_pile(game: &Game, width: f32, height: f32, compact: bool, progress: &progress::Progress) {
    let pile = pile_card_rect(width, height);
    let center_x = pile.x + pile.w / 2.0;
    if !compact {
        draw_centered_text(
            "SPELHÖG - KLICKA FÖR ATT SPELA",
            center_x,
            pile.y - 14.0,
            18,
            LIGHTGRAY,
        );
    }
    if let Some(card) = game.pile().last() {
        card_renderer::draw_face_with_theme(
            pile,
            *card,
            CardStyle::default(),
            progress.front_theme,
        );
        if !compact {
            draw_centered_text(
                &format!("{} kort i spelhögen", game.pile().len()),
                center_x,
                pile.y + pile.h + 22.0,
                18,
                WHITE,
            );
        }
    } else {
        draw_rectangle(
            pile.x,
            pile.y,
            pile.w,
            pile.h,
            Color::new(0.03, 0.2, 0.14, 1.0),
        );
        draw_rectangle_lines(pile.x, pile.y, pile.w, pile.h, 2.0, LIGHTGRAY);
        draw_centered_text("TOM", center_x, pile.y + 78.0, 18, LIGHTGRAY);
    }
}

fn pile_card_rect(width: f32, height: f32) -> Rect {
    let compact = compact_layout(width, height);
    let card_height = card_height_for_screen(width, height);
    let card_width = card_height * CARD_WIDTH / CARD_HEIGHT;
    let (center_x, center_y) = if compact && height < 520.0 {
        (
            card_width / 2.0 + 12.0,
            player_cards_y(width, height, card_height) + card_height / 2.0,
        )
    } else if compact {
        let grid_bottom = compact_player_grid_bottom(width, 5);
        let center_y = (height * 0.28).max(grid_bottom + 8.0 + card_height / 2.0);
        (width * 0.82, center_y)
    } else {
        (width * 0.82, height * 0.36)
    };
    Rect::new(
        center_x - card_width / 2.0,
        center_y - card_height * 0.34,
        card_width,
        card_height,
    )
}

fn draw_player_cards(
    game: &Game,
    cursor: usize,
    selected: &[usize],
    width: f32,
    height: f32,
    progress: &progress::Progress,
) {
    let compact = compact_layout(width, height);
    let card_height = card_height_for_screen(width, height);
    let phase = game.player_phase(0).unwrap_or(PlayerPhase::FaceDown);
    let cards = player_cards(game, phase);
    let count = player_card_count(game, phase);
    let title = match phase {
        PlayerPhase::Hand => "DIN HAND",
        PlayerPhase::FaceUp => "DINA ÖPPNA KORT",
        PlayerPhase::FaceDown => "DINA BLINDA KORT",
    };
    let start_y = player_cards_y(width, height, card_height);
    if !compact {
        draw_text(title, 28.0, start_y - 16.0, 20.0, GOLD);
    }
    let (card_width, card_gap, start_x) = card_layout(count, width, height, card_height, compact);
    let display_order = player_display_order(game, phase);

    for (index, card_index) in display_order.iter().copied().enumerate() {
        let x = start_x + index as f32 * (card_width + card_gap);
        let is_selected = selected.contains(&card_index);
        let y = start_y;
        if phase == PlayerPhase::FaceDown {
            card_renderer::draw_back_with_theme(
                Rect::new(x, y, card_width, card_height),
                index == cursor,
                progress.back_theme,
            );
            draw_text(
                format!("{}", index + 1),
                x + card_width / 2.0 - 5.0,
                y + card_height + if compact { 16.0 } else { 19.0 },
                if compact { 12.0 } else { 16.0 },
                LIGHTGRAY,
            );
        } else if let Some(card) = cards.get(card_index) {
            card_renderer::draw_face_with_theme(
                Rect::new(x, y, card_width, card_height),
                *card,
                CardStyle {
                    selected: is_selected,
                    focused: index == cursor,
                },
                progress.front_theme,
            );
            draw_text(
                format!("{}", index + 1),
                x + card_width / 2.0 - 5.0,
                y + card_height + if compact { 16.0 } else { 19.0 },
                if compact { 12.0 } else { 16.0 },
                WHITE,
            );
        }
    }
    if phase != PlayerPhase::FaceDown {
        draw_action_buttons(game, phase, width, height, selected.len());
    }
}

#[derive(Clone, Copy)]
struct ActionButtonRects {
    play: Rect,
    draw: Rect,
    pickup: Rect,
}

fn action_button_rects(width: f32, height: f32) -> ActionButtonRects {
    if compact_layout(width, height) {
        let margin = 10.0;
        let gap = 6.0;
        let button_width = (width - margin * 2.0 - gap * 2.0) / 3.0;
        let y = height - COMPACT_ACTION_BUTTON_HEIGHT - 24.0;
        return ActionButtonRects {
            play: Rect::new(margin, y, button_width, COMPACT_ACTION_BUTTON_HEIGHT),
            draw: Rect::new(
                margin + button_width + gap,
                y,
                button_width,
                COMPACT_ACTION_BUTTON_HEIGHT,
            ),
            pickup: Rect::new(
                margin + (button_width + gap) * 2.0,
                y,
                button_width,
                COMPACT_ACTION_BUTTON_HEIGHT,
            ),
        };
    }

    let scale = ((width - 24.0) / 464.0).clamp(0.0, 1.0);
    let play_width = 180.0 * scale;
    let other_width = 130.0 * scale;
    let gap = 12.0 * scale;
    let total_width = play_width + other_width * 2.0 + gap * 2.0;
    let left = ((width - total_width) / 2.0).max(12.0);
    let y = height - 101.0;
    ActionButtonRects {
        play: Rect::new(left, y, play_width, ACTION_BUTTON_HEIGHT),
        draw: Rect::new(
            left + play_width + gap,
            y,
            other_width,
            ACTION_BUTTON_HEIGHT,
        ),
        pickup: Rect::new(
            left + play_width + gap + other_width + gap,
            y,
            other_width,
            ACTION_BUTTON_HEIGHT,
        ),
    }
}

fn draw_action_buttons(
    game: &Game,
    phase: PlayerPhase,
    width: f32,
    height: f32,
    selected_count: usize,
) {
    let buttons = action_button_rects(width, height);
    let compact = compact_layout(width, height);
    let play_text = if compact {
        "SPELA"
    } else if selected_count == 0 {
        "MARKERA KORT"
    } else {
        "SPELA MARKERADE"
    };
    draw_button(buttons.play, play_text, selected_count > 0);

    if phase == PlayerPhase::Hand && game.draw_pile_size() > 0 {
        draw_button(
            buttons.draw,
            if compact { "DRA" } else { "DRA + SPELA" },
            true,
        );
    } else {
        draw_button(
            buttons.draw,
            if compact { "DRA" } else { "SLUT PÅ KORT" },
            false,
        );
    }

    if !game.pile().is_empty() && phase != PlayerPhase::FaceDown {
        draw_button(
            buttons.pickup,
            if compact { "PLOCKA" } else { "PLOCKA UPP" },
            true,
        );
    } else {
        draw_button(
            buttons.pickup,
            if compact { "PLOCKA" } else { "PLOCKA UPP" },
            false,
        );
    }
}

fn draw_button(rect: Rect, label: &str, enabled: bool) {
    let fill = if enabled {
        Color::new(0.18, 0.55, 0.35, 1.0)
    } else {
        Color::new(0.12, 0.19, 0.16, 1.0)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if enabled { 3.0 } else { 2.0 },
        if enabled { GOLD } else { DARKGRAY },
    );
    let base_font_size: u16 = if rect.h >= COMPACT_ACTION_BUTTON_HEIGHT {
        20
    } else {
        16
    };
    let text_width = measure_text(label, None, base_font_size, 1.0).width;
    let font_size = (f32::from(base_font_size) * (rect.w * 0.9 / text_width))
        .clamp(14.0, f32::from(base_font_size));
    draw_text(
        label,
        rect.x + (rect.w - text_width * font_size / f32::from(base_font_size)) / 2.0,
        rect.y + (rect.h + font_size * 0.62) / 2.0,
        font_size,
        if enabled { WHITE } else { LIGHTGRAY },
    );
}

fn draw_button_contains(rect: Rect, x: f32, y: f32) -> bool {
    x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h
}

fn play_button_contains(x: f32, y: f32, width: f32, height: f32) -> bool {
    draw_button_contains(action_button_rects(width, height).play, x, y)
}

fn draw_button_contains_at(x: f32, y: f32, width: f32, height: f32) -> bool {
    draw_button_contains(action_button_rects(width, height).draw, x, y)
}

fn pickup_button_contains(x: f32, y: f32, width: f32, height: f32) -> bool {
    draw_button_contains(action_button_rects(width, height).pickup, x, y)
}

fn pile_target_contains(x: f32, y: f32, width: f32, height: f32) -> bool {
    let pile = pile_card_rect(width, height);
    x >= pile.x && x <= pile.x + pile.w && y >= pile.y && y <= pile.y + pile.h
}

fn card_layout(
    count: usize,
    screen_width: f32,
    screen_height: f32,
    card_height: f32,
    compact: bool,
) -> (f32, f32, f32) {
    let natural_width = card_height * CARD_WIDTH / CARD_HEIGHT;
    let reserve_left = if compact && screen_height < 520.0 {
        natural_width + 36.0
    } else {
        0.0
    };
    let available_width =
        (screen_width - reserve_left - if compact { 24.0 } else { 44.0 }).max(1.0);
    if count == 0 {
        return (natural_width, CARD_GAP, reserve_left + 12.0);
    }
    let count_f = count as f32;
    let card_width = natural_width.min(available_width);
    let preferred_gap = CARD_GAP * card_height / CARD_HEIGHT;
    let gap =
        if count > 1 && count_f * (card_width + preferred_gap) - preferred_gap > available_width {
            (available_width - card_width * count_f) / (count_f - 1.0)
        } else if count > 1 {
            preferred_gap
        } else {
            0.0
        };
    let total_width = count_f * card_width + (count_f - 1.0).max(0.0) * gap;
    let start_x = (reserve_left + (screen_width - reserve_left - total_width) / 2.0).max(12.0);
    (card_width, gap, start_x)
}

fn compact_player_grid_bottom(width: f32, player_count: usize) -> f32 {
    let columns = ((width / 170.0).floor() as usize)
        .max(1)
        .min(player_count.max(1));
    let rows = player_count.div_ceil(columns);
    42.0 + rows as f32 * 42.0 + rows.saturating_sub(1) as f32 * 6.0
}

fn draw_controls(game: &Game, status: &str, width: f32, height: f32, compact: bool) {
    if compact {
        if status != INITIAL_STATUS && status.starts_with("Kunde inte") {
            let font_size = 12;
            let text_width = measure_text(status, None, font_size, 1.0).width;
            let scale = ((width - 16.0) / text_width).min(1.0);
            let fitted_width = text_width * scale;
            draw_text(
                status,
                (width - fitted_width) / 2.0,
                action_button_rects(width, height).play.y - 8.0,
                f32::from(font_size) * scale,
                YELLOW,
            );
        }
        return;
    }
    let panel_y = height - 43.0;
    draw_rectangle(0.0, panel_y - 6.0, width, 52.0, PANEL_COLOR);
    draw_text(status, 20.0, panel_y + 12.0, 18.0, YELLOW);
    let phase = game.player_phase(0).unwrap_or(PlayerPhase::FaceDown);
    let controls = match phase {
        PlayerPhase::Hand => {
            "PILTANGENTER: FLYTTA  MELLANSLAG: MARKERA  RETUR: SPELA  D: DRA  P: PLOCKA UPP  Q: AVSLUTA"
        }
        PlayerPhase::FaceUp => {
            "PILTANGENTER: FLYTTA  MELLANSLAG: MARKERA  RETUR: SPELA  P: PLOCKA UPP  Q: AVSLUTA"
        }
        PlayerPhase::FaceDown => "PILTANGENTER: FLYTTA  RETUR: VÄND  Q: AVSLUTA",
    };
    draw_text(controls, 20.0, panel_y + 34.0, 14.0, LIGHTGRAY);
}

fn compact_layout(width: f32, height: f32) -> bool {
    width < 700.0 || height < 520.0
}

fn card_height_for_screen(width: f32, height: f32) -> f32 {
    if compact_layout(width, height) {
        compact_card_height(height)
    } else {
        (CARD_HEIGHT * height / 760.0).clamp(120.0, CARD_HEIGHT)
    }
}

fn compact_card_height(height: f32) -> f32 {
    (height * 0.24).clamp(86.0, 128.0)
}

fn player_cards_y(width: f32, height: f32, card_height: f32) -> f32 {
    let bottom_space = if compact_layout(width, height) {
        if height < 520.0 { 108.0 } else { 190.0 }
    } else {
        154.0
    };
    height - card_height - bottom_space
}

fn player_display_order(game: &Game, phase: PlayerPhase) -> Vec<usize> {
    let count = player_card_count(game, phase);
    if phase != PlayerPhase::Hand {
        return (0..count).collect();
    }

    let mut indices = (0..count).collect::<Vec<_>>();
    if let Some(hand) = game.hand(0) {
        indices.sort_by_key(|index| vändtia_rank_sort_key(hand[*index].rank));
    }
    indices
}

fn vändtia_rank_sort_key(rank: Rank) -> u8 {
    match rank {
        Rank::Three => 0,
        Rank::Four => 1,
        Rank::Five => 2,
        Rank::Six => 3,
        Rank::Seven => 4,
        Rank::Eight => 5,
        Rank::Nine => 6,
        Rank::Jack => 7,
        Rank::Queen => 8,
        Rank::King => 9,
        Rank::Ace => 10,
        Rank::Two => 11,
        Rank::Ten => 12,
    }
}

fn card_at_position(
    mouse_x: f32,
    mouse_y: f32,
    count: usize,
    width: f32,
    height: f32,
) -> Option<usize> {
    let compact = compact_layout(width, height);
    let card_height = card_height_for_screen(width, height);
    let (card_width, gap, start_x) = card_layout(count, width, height, card_height, compact);
    let start_y = player_cards_y(width, height, card_height);
    (0..count).rev().find(|index| {
        let x = start_x + *index as f32 * (card_width + gap);
        let y = start_y;
        mouse_x >= x && mouse_x <= x + card_width && mouse_y >= y && mouse_y <= y + card_height
    })
}

fn player_cards(game: &Game, phase: PlayerPhase) -> &[Card] {
    match phase {
        PlayerPhase::Hand => game.hand(0).unwrap_or(&[]),
        PlayerPhase::FaceUp => game.face_up(0).unwrap_or(&[]),
        PlayerPhase::FaceDown => &[],
    }
}

fn player_card_count(game: &Game, phase: PlayerPhase) -> usize {
    match phase {
        PlayerPhase::Hand => game.hand(0).map_or(0, <[Card]>::len),
        PlayerPhase::FaceUp => game.face_up(0).map_or(0, <[Card]>::len),
        PlayerPhase::FaceDown => game.face_down_count(0).unwrap_or(0),
    }
}

fn selected_indices(selected: &[usize], cursor: usize, phase: PlayerPhase) -> Vec<usize> {
    if phase == PlayerPhase::FaceDown {
        return vec![cursor];
    }
    selected.to_vec()
}

fn toggle_selection(selected: &mut Vec<usize>, index: usize) {
    if let Some(position) = selected
        .iter()
        .position(|selected_index| *selected_index == index)
    {
        selected.remove(position);
    } else {
        selected.push(index);
        selected.sort_unstable();
    }
}

fn format_outcome(actor: &str, outcome: &TurnOutcome) -> String {
    let mut message = if let Some(card) = outcome.drawn {
        if outcome.played.is_some() {
            format!(
                "{actor}drog och spelade {}.",
                card_renderer::format_card(card)
            )
        } else {
            format!(
                "{actor}drog {} och tog upp spelhögen.",
                card_renderer::format_card(card)
            )
        }
    } else if !outcome.played_cards.is_empty() {
        let cards = outcome
            .played_cards
            .iter()
            .map(|card| card_renderer::format_card(*card))
            .collect::<Vec<_>>()
            .join(" och ");
        format!("{actor}spelade {cards}.")
    } else if let Some(card) = outcome.played {
        format!("{actor}vände {}.", card_renderer::format_card(card))
    } else {
        format!("{actor}plockade upp spelhögen.")
    };
    if outcome.burned {
        message.push_str(" Spelhögen tömdes - en tur till!");
    } else if outcome.extra_turn {
        message.push_str(" En tur till!");
    }
    if outcome.picked_up > 0 && outcome.drawn.is_none() {
        message.push_str(&format!(" {} kort plockades upp.", outcome.picked_up));
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_text_converts_non_ascii_glyphs_before_drawing() {
        assert_eq!(
            ascii_game_text("VÄNDTIA • 1–5: välj → spela"),
            "VANDTIA * 1-5: valj -> spela"
        );
        assert_eq!(ascii_game_text("kort 🂡"), "kort ?");
    }

    #[test]
    fn enter_uses_only_toggled_cards() {
        assert_eq!(
            selected_indices(&[], 0, PlayerPhase::Hand),
            Vec::<usize>::new()
        );
        assert_eq!(selected_indices(&[1, 2], 0, PlayerPhase::Hand), vec![1, 2]);
        assert_eq!(selected_indices(&[], 2, PlayerPhase::FaceDown), vec![2]);
    }

    #[test]
    fn mouse_position_selects_card_bounds() {
        assert_eq!(card_at_position(250.0, 530.0, 3, 800.0, 760.0), Some(0));
        assert_eq!(card_at_position(340.0, 530.0, 3, 800.0, 760.0), None);
    }

    #[test]
    fn overlapping_cards_remain_individually_selectable() {
        let count = 30;
        let width = 800.0;
        let height = 760.0;
        let (card_width, gap, start_x) = card_layout(count, width, height, CARD_HEIGHT, false);
        let step = card_width + gap;
        let y = player_cards_y(width, height, CARD_HEIGHT) + CARD_HEIGHT / 2.0;

        assert!(step > 0.0);
        for index in 0..count {
            let x = start_x + index as f32 * step;
            let click_x = if index + 1 == count {
                x + 5.0
            } else {
                x + step / 2.0
            };
            assert_eq!(
                card_at_position(click_x, y, count, width, height),
                Some(index)
            );
        }
    }

    #[test]
    fn action_button_hit_regions_match_their_layout() {
        for width in [360.0, 800.0, 1100.0] {
            let height = 760.0;
            let buttons = action_button_rects(width, height);
            let center_y = buttons.play.y + buttons.play.h / 2.0;

            assert!(play_button_contains(
                buttons.play.x + buttons.play.w / 2.0,
                center_y,
                width,
                height
            ));
            assert!(draw_button_contains_at(
                buttons.draw.x + buttons.draw.w / 2.0,
                center_y,
                width,
                height
            ));
            assert!(pickup_button_contains(
                buttons.pickup.x + buttons.pickup.w / 2.0,
                center_y,
                width,
                height
            ));
            assert!(!play_button_contains(
                buttons.draw.x + buttons.draw.w / 2.0,
                center_y,
                width,
                height
            ));
        }
    }

    #[test]
    fn compact_layout_gives_mobile_controls_and_cards_readable_dimensions() {
        for (width, height) in [(390.0, 844.0), (844.0, 390.0)] {
            assert!(compact_layout(width, height));

            let buttons = action_button_rects(width, height);
            assert_eq!(buttons.play.h, COMPACT_ACTION_BUTTON_HEIGHT);
            assert_eq!(buttons.draw.h, COMPACT_ACTION_BUTTON_HEIGHT);
            assert_eq!(buttons.pickup.h, COMPACT_ACTION_BUTTON_HEIGHT);
            assert!(buttons.play.w >= 80.0);
            assert!(buttons.draw.w >= 80.0);
            assert!(buttons.pickup.w >= 80.0);

            let card_height = card_height_for_screen(width, height);
            assert!(card_height >= 86.0);
            let (card_width, _, _) = card_layout(3, width, height, card_height, true);
            assert!(card_width >= 60.0);
            assert!(
                player_cards_y(width, height, card_height) + card_height + 16.0 < buttons.play.y
            );
        }
    }

    #[test]
    fn setup_controls_fit_narrow_mobile_widths() {
        for width in [280.0, 320.0, 390.0] {
            let (minus, plus) = setup_player_buttons(width, 700.0);
            let start = setup_start_button(width, 700.0);

            assert!(minus.x >= 0.0);
            assert!(plus.x + plus.w <= width);
            assert!(minus.w >= 64.0);
            assert!(start.x >= 0.0);
            assert!(start.x + start.w <= width);
            assert!(start.h >= 64.0);
        }
    }

    #[test]
    fn setup_screen_fits_short_landscape_viewports() {
        let height = 390.0;
        let center_y = setup_player_selector_y(height);
        let (_, plus) = setup_player_buttons(844.0, height);
        let start = setup_start_button(844.0, height);

        assert!(plus.y + plus.h < center_y + 50.0);
        assert!(center_y + 50.0 < start.y);
    }

    #[test]
    fn card_hit_regions_follow_responsive_card_dimensions() {
        let (width, height) = (390.0, 844.0);
        let card_height = card_height_for_screen(width, height);
        let (card_width, gap, start_x) = card_layout(3, width, height, card_height, true);
        let card_y = player_cards_y(width, height, card_height);

        assert_eq!(
            card_at_position(
                start_x + card_width / 2.0,
                card_y + card_height / 2.0,
                3,
                width,
                height,
            ),
            Some(0)
        );
        assert_eq!(
            card_at_position(
                start_x + card_width + gap / 2.0,
                card_y + card_height / 2.0,
                3,
                width,
                height,
            ),
            None
        );
    }

    #[test]
    fn opponent_grid_keeps_all_players_in_the_reserved_area() {
        assert_eq!(opponent_columns(4, 990.0), 4);
        assert_eq!(opponent_columns(4, 1100.0), 4);
        assert_eq!(opponent_columns(4, 500.0), 1);
        assert_eq!(4_usize.div_ceil(opponent_columns(4, 1100.0)), 1);
        assert_eq!(4_usize.div_ceil(opponent_columns(4, 500.0)), 4);
    }

    #[test]
    fn setup_controls_stay_within_the_player_selector() {
        let (minus, plus) = setup_player_buttons(1100.0, 760.0);
        let start = setup_start_button(1100.0, 760.0);
        assert_eq!(minus.y, plus.y);
        assert!(minus.y + minus.h < start.y);
        assert!(76.0 + 140.0 < player_cards_y(1100.0, 760.0, CARD_HEIGHT));
    }

    #[test]
    fn game_menu_buttons_fit_narrow_and_short_screens() {
        for (width, height) in [(280.0, 390.0), (390.0, 844.0), (1100.0, 760.0)] {
            let vändtia = vandtia_menu_button(width, height);
            let sjuan = sjuan_menu_option_rect(width, height);
            let patience = solitaire_menu_option_rect(width, height);
            let shop = shop_menu_option_rect(width, height);
            let gallery = card_gallery_menu_option_rect(width, height);
            let buttons = [vändtia, sjuan, patience, shop, gallery];
            for button in buttons {
                assert!(button.x >= 0.0);
                assert!(button.y >= 0.0);
                assert!(button.x + button.w <= width);
                assert!(button.y + button.h < height - 20.0);
            }
            for first in 0..buttons.len() {
                for second in first + 1..buttons.len() {
                    assert!(
                        buttons[first].y + buttons[first].h <= buttons[second].y,
                        "menu buttons overlap at {width}x{height}"
                    );
                }
            }
        }
    }

    #[test]
    fn shop_choices_are_clickable_and_fit_narrow_screens() {
        for (width, height) in [(280.0, 390.0), (390.0, 844.0), (1100.0, 760.0)] {
            for (theme, rect) in shop_choice_rects(width, height).into_iter().enumerate() {
                assert!(rect.x >= 0.0);
                assert!(rect.x + rect.w <= width);
                assert!(rect.y + rect.h < height);
                assert_eq!(
                    shop_choice_at(rect.x + rect.w / 2.0, rect.y + rect.h / 2.0, width, height),
                    Some(theme)
                );
            }
            for (tab, rect) in shop_tab_buttons(width).into_iter().enumerate() {
                assert!(rect.x >= 0.0);
                assert!(rect.x + rect.w <= width);
                assert_eq!(
                    shop_tab_at(rect.x + rect.w / 2.0, rect.y + rect.h / 2.0, width),
                    Some(tab == 0)
                );
            }
        }
    }

    #[test]
    fn card_gallery_sizes_are_distinct_and_grid_stays_inside_viewport() {
        let (width, height) = (1100.0, 760.0);
        let sizes = (0..3)
            .map(|size| card_gallery_layout(width, height, size))
            .collect::<Vec<_>>();
        assert!(sizes[0].0 < sizes[1].0);
        assert!(sizes[1].0 < sizes[2].0);

        for (size, (card_width, card_height, gap)) in sizes.into_iter().enumerate() {
            assert!(card_width > 0.0);
            assert!(card_height > 0.0);
            assert!(gap >= 0.0);
            let selector = card_gallery_size_buttons(width, height);
            let start_y = selector[0].y + selector[0].h + 18.0;
            let last_card_bottom = start_y + 3.0 * (card_height + 18.0) + card_height;
            assert!(last_card_bottom < height - 20.0, "gallery size {size}");
            assert!(14.0 + 12.0 * (card_width + gap) + card_width < width);
        }
    }

    #[test]
    fn solitaire_board_and_menu_button_do_not_overlap_on_short_screens() {
        for (width, height) in [(844.0, 390.0), (390.0, 844.0), (1100.0, 760.0)] {
            let layout = solitaire_board_layout(width, height);
            let menu = solitaire_menu_button(width, height);
            assert!(layout.stock.y >= menu.y + menu.h);
            assert!(layout.columns[6] + layout.card_width <= width);
            assert!(layout.card_width >= 34.0);
        }
    }

    #[test]
    fn game_over_menu_button_is_centered_and_clickable() {
        let button = menu_button_rect(1100.0, 760.0);
        assert_eq!(button.x + button.w / 2.0, 550.0);
        assert_eq!(button.y + button.h / 2.0, 498.2);
        assert!(draw_button_contains(
            button,
            button.x + button.w / 2.0,
            button.y + button.h / 2.0
        ));
        assert!(!draw_button_contains(button, button.x - 1.0, button.y));
    }

    #[test]
    fn player_table_grid_includes_every_player() {
        assert_eq!(2_usize.div_ceil(opponent_columns(2, 1100.0)), 1);
        assert_eq!(5_usize.div_ceil(opponent_columns(5, 1100.0)), 2);
        assert_eq!(opponent_columns(5, 1100.0), 4);
    }

    #[test]
    fn table_stacks_show_face_up_cards_over_remaining_face_down_cards() {
        let face_up = [
            Card {
                rank: Rank::Jack,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Clubs,
            },
        ];
        assert_eq!(
            table_stack_layers(&face_up, 3),
            [
                (true, Some(face_up[0])),
                (true, Some(face_up[1])),
                (true, None),
            ]
        );
        assert_eq!(
            table_stack_layers(&[], 2),
            [(true, None), (true, None), (false, None)]
        );
    }

    #[test]
    fn compact_player_tables_keep_face_down_cards_visible() {
        assert_eq!(
            table_stack_layers(&[], 3),
            [(true, None), (true, None), (true, None)]
        );
    }

    #[test]
    fn compact_portrait_pile_stays_between_player_tables_and_hand() {
        let (width, height) = (390.0, 711.0);
        let pile = pile_card_rect(width, height);
        let cards_y = player_cards_y(width, height, card_height_for_screen(width, height));

        assert!(pile.y >= compact_player_grid_bottom(width, 5));
        assert!(pile.y + pile.h < cards_y);
    }

    #[test]
    fn compact_landscape_pile_sits_left_of_the_hand() {
        let (width, height) = (844.0, 390.0);
        let pile = pile_card_rect(width, height);
        let card_height = card_height_for_screen(width, height);
        let (card_width, _, start_x) = card_layout(3, width, height, card_height, true);

        assert!(pile.x + pile.w < start_x);
        assert!(pile.y >= 0.0);
        assert!(pile.y + pile.h <= height);
        assert!(card_width >= 60.0);
    }

    #[test]
    fn pile_hit_region_matches_its_draw_position() {
        for (width, height) in [(500.0, 600.0), (990.0, 700.0), (1100.0, 760.0)] {
            let pile = pile_card_rect(width, height);
            assert!(pile_target_contains(
                pile.x + pile.w / 2.0,
                pile.y + pile.h / 2.0,
                width,
                height
            ));
            assert!(!pile_target_contains(pile.x - 1.0, pile.y, width, height));
        }
    }

    #[test]
    fn hand_sort_uses_rank_without_grouping_suits_and_places_two_and_ten_last() {
        let cards = [
            Card {
                rank: Rank::Ten,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Ace,
                suit: Suit::Clubs,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Clubs,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Hearts,
            },
        ];
        let mut sorted = cards;
        sorted.sort_by_key(|card| vändtia_rank_sort_key(card.rank));
        assert_eq!(
            sorted,
            [
                Card {
                    rank: Rank::Three,
                    suit: Suit::Hearts
                },
                Card {
                    rank: Rank::Ace,
                    suit: Suit::Clubs
                },
                Card {
                    rank: Rank::Two,
                    suit: Suit::Clubs
                },
                Card {
                    rank: Rank::Ten,
                    suit: Suit::Hearts
                },
            ]
        );
    }
}
