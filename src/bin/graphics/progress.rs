use std::{error::Error, fmt};

pub const FRONT_PRICES: [u32; 6] = [0, 40, 60, 90, 120, 150];
pub const BACK_PRICES: [u32; 6] = [0, 30, 50, 75, 100, 125];
#[cfg(target_arch = "wasm32")]
const PROGRESS_CAPACITY: usize = 96;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    pub coins: u32,
    pub front_theme: usize,
    pub back_theme: usize,
    owned_front: u8,
    owned_back: u8,
}

#[derive(Debug)]
pub struct ProgressError(String);

impl fmt::Display for ProgressError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for ProgressError {}

impl Default for Progress {
    fn default() -> Self {
        Self {
            coins: 0,
            front_theme: 0,
            back_theme: 0,
            owned_front: 1,
            owned_back: 1,
        }
    }
}

impl Progress {
    pub fn owns_front(&self, theme: usize) -> bool {
        theme < FRONT_PRICES.len() && self.owned_front & (1 << theme) != 0
    }

    pub fn owns_back(&self, theme: usize) -> bool {
        theme < BACK_PRICES.len() && self.owned_back & (1 << theme) != 0
    }

    pub fn buy_or_equip_front(&mut self, theme: usize) -> Result<bool, &'static str> {
        buy_or_equip(
            theme,
            &mut self.coins,
            &mut self.owned_front,
            &mut self.front_theme,
            &FRONT_PRICES,
        )
    }

    pub fn buy_or_equip_back(&mut self, theme: usize) -> Result<bool, &'static str> {
        buy_or_equip(
            theme,
            &mut self.coins,
            &mut self.owned_back,
            &mut self.back_theme,
            &BACK_PRICES,
        )
    }

    pub fn award(&mut self, coins: u32) {
        self.coins = self.coins.saturating_add(coins);
    }

    fn encode(&self) -> String {
        format!(
            "1\n{}\n{}\n{}\n{}\n{}",
            self.coins, self.front_theme, self.back_theme, self.owned_front, self.owned_back
        )
    }

    fn decode(saved: &str) -> Result<Self, ProgressError> {
        let values = saved.lines().map(str::trim).collect::<Vec<_>>();
        if values.len() != 6 || values[0] != "1" {
            return Err(ProgressError("Sparfilen har ett okänt format.".to_owned()));
        }
        let parse = |index: usize| {
            values[index].parse::<u32>().map_err(|error| {
                ProgressError(format!("Sparfilen innehåller ett ogiltigt värde: {error}"))
            })
        };
        let coins = parse(1)?;
        let front_theme = parse(2)? as usize;
        let back_theme = parse(3)? as usize;
        let owned_front = u8::try_from(parse(4)?)
            .map_err(|error| ProgressError(format!("Ogiltiga framsidor i sparfilen: {error}")))?;
        let owned_back = u8::try_from(parse(5)?)
            .map_err(|error| ProgressError(format!("Ogiltiga baksidor i sparfilen: {error}")))?;
        if front_theme >= FRONT_PRICES.len()
            || back_theme >= BACK_PRICES.len()
            || owned_front & 1 == 0
            || owned_back & 1 == 0
            || owned_front & !0b11_1111 != 0
            || owned_back & !0b11_1111 != 0
            || owned_front & (1 << front_theme) == 0
            || owned_back & (1 << back_theme) == 0
        {
            return Err(ProgressError(
                "Sparfilen innehåller en ogiltig kortstil.".to_owned(),
            ));
        }
        Ok(Self {
            coins,
            front_theme,
            back_theme,
            owned_front,
            owned_back,
        })
    }
}

fn buy_or_equip(
    theme: usize,
    coins: &mut u32,
    owned: &mut u8,
    selected: &mut usize,
    prices: &[u32; 6],
) -> Result<bool, &'static str> {
    if theme >= prices.len() {
        return Err("Den kortstilen finns inte.");
    }
    if *owned & (1_u8 << theme) == 0 {
        if *coins < prices[theme] {
            return Err("Du har inte tillräckligt med mynt.");
        }
        *coins -= prices[theme];
        *owned |= 1_u8 << theme;
    }
    let changed = *selected != theme;
    *selected = theme;
    Ok(changed)
}

pub fn load() -> Result<Progress, Box<dyn Error>> {
    let saved = load_saved()?;
    let progress = match saved {
        Some(saved) => Progress::decode(&saved)?,
        None => Progress::default(),
    };
    macroquad::experimental::collections::storage::store(progress.clone());
    Ok(progress)
}

pub fn save(progress: &Progress) -> Result<(), Box<dyn Error>> {
    macroquad::experimental::collections::storage::store(progress.clone());
    save_string(&progress.encode())?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn load_saved() -> Result<Option<String>, Box<dyn Error>> {
    let path = save_path();
    match std::fs::read_to_string(path) {
        Ok(saved) => Ok(Some(saved)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Box::new(error)),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_string(saved: &str) -> Result<(), Box<dyn Error>> {
    let path = save_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, saved)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn save_path() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(std::path::PathBuf::from);
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(std::path::PathBuf::from)
                .map(|home| home.join(".local/share"))
        });

    base.unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    })
    .join("vandtia")
    .join("progress.dat")
}

#[cfg(target_arch = "wasm32")]
fn load_saved() -> Result<Option<String>, Box<dyn Error>> {
    let mut buffer = [0_u8; PROGRESS_CAPACITY];
    let length = unsafe { load_progress(buffer.as_mut_ptr(), buffer.len()) };
    match length {
        -1 => Ok(None),
        -2 => Err(Box::new(ProgressError(
            "Kunde inte läsa sparad progress från webbläsaren.".to_owned(),
        ))),
        length if length >= 0 && length as usize <= buffer.len() => {
            Ok(Some(String::from_utf8(buffer[..length as usize].to_vec())?))
        }
        _ => Err(Box::new(ProgressError(
            "Sparad progress från webbläsaren är för stor.".to_owned(),
        ))),
    }
}

#[cfg(target_arch = "wasm32")]
fn save_string(saved: &str) -> Result<(), Box<dyn Error>> {
    let result = unsafe { store_progress(saved.as_ptr(), saved.len()) };
    if result == 0 {
        Ok(())
    } else {
        Err(Box::new(ProgressError(
            "Kunde inte spara progress i webbläsaren.".to_owned(),
        )))
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn load_progress(destination: *mut u8, capacity: usize) -> i32;
    fn store_progress(data: *const u8, length: usize) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purchase_deducts_coins_and_equipping_is_free() {
        let mut progress = Progress::default();
        progress.award(40);

        assert!(progress.buy_or_equip_front(1).unwrap());
        assert_eq!(progress.coins, 0);
        assert!(!progress.buy_or_equip_front(1).unwrap());
        assert_eq!(progress.coins, 0);
    }

    #[test]
    fn cannot_buy_without_enough_coins_or_select_unknown_style() {
        let mut progress = Progress::default();
        assert_eq!(
            progress.buy_or_equip_back(1),
            Err("Du har inte tillräckligt med mynt.")
        );
        assert_eq!(
            progress.buy_or_equip_front(3),
            Err("Du har inte tillräckligt med mynt.")
        );
        assert_eq!(
            progress.buy_or_equip_front(6),
            Err("Den kortstilen finns inte.")
        );
        assert_eq!(progress.front_theme, 0);
    }

    #[test]
    fn progress_round_trips_and_rejects_unowned_equipped_styles() {
        let mut progress = Progress::default();
        progress.award(300);
        progress.buy_or_equip_back(5).unwrap();
        progress.buy_or_equip_front(4).unwrap();
        assert_eq!(Progress::decode(&progress.encode()).unwrap(), progress);
        assert!(Progress::decode("1\n100\n0\n2\n1\n1").is_err());
    }

    #[test]
    fn six_styles_can_be_owned_and_equipped() {
        let mut progress = Progress::default();
        progress.award(900);
        for theme in 1..6 {
            progress.buy_or_equip_front(theme).unwrap();
            progress.buy_or_equip_back(theme).unwrap();
        }
        assert_eq!(progress.front_theme, 5);
        assert_eq!(progress.back_theme, 5);
        assert!(progress.owns_front(4));
        assert!(progress.owns_back(4));
    }
}
