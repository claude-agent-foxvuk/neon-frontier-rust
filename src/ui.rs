use bevy::prelude::{Component, FontSize, FontSource, TextFont, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuOption {
    NewGame,
    LoadGame,
    Credits,
}

pub const MENU_OPTIONS: [MenuOption; 3] = [
    MenuOption::NewGame,
    MenuOption::LoadGame,
    MenuOption::Credits,
];

pub const IN_GAME_HELP: &str = "WASD — движение / газ • Shift — бег\nSpace — прыжок • E — машина / выход\nF — удар по пешеходам • B — Bloom\nF5 — сохранить • F9 — загрузить";

pub const SHORT_CREDITS: &str = "Сделано FoxVuk и Claude Opus 5.5";

const MENU_TEXT_LEFT: f32 = 24.0;
const MENU_TEXT_RIGHT: f32 = 540.0;
const MENU_FIRST_ROW_TOP: f32 = 140.0;
const MENU_ROW_HEIGHT: f32 = 24.0;
const HUD_MAX_LINE_CHARS: usize = 48;

#[derive(Component)]
pub struct Hud;

pub fn hud_font() -> TextFont {
    TextFont {
        font: FontSource::families("'Segoe UI', Arial, sans-serif"),
        font_size: FontSize::Px(19.0),
        ..Default::default()
    }
}

fn option_label(option: MenuOption) -> &'static str {
    match option {
        MenuOption::NewGame => "Новая игра",
        MenuOption::LoadGame => "Загрузить сохранение",
        MenuOption::Credits => "Кредитс",
    }
}

pub fn menu_index_at_cursor(position: Vec2) -> Option<usize> {
    if !position.is_finite()
        || position.x < MENU_TEXT_LEFT
        || position.x > MENU_TEXT_RIGHT
        || position.y < MENU_FIRST_ROW_TOP
    {
        return None;
    }
    let index = ((position.y - MENU_FIRST_ROW_TOP) / MENU_ROW_HEIGHT).floor() as usize;
    (index < MENU_OPTIONS.len()).then_some(index)
}

pub fn main_menu_text(selected: usize, credits_open: bool, notice: &str) -> String {
    if credits_open {
        let mut text = String::from("NEON FRONTIER\n\nКРЕДИТС\n\n");
        text.push_str("Открытый мир на Rust и Bevy для Windows 11\n");
        text.push_str(SHORT_CREDITS);
        text.push_str("\n\n");
        text.push_str(IN_GAME_HELP);
        text.push_str("\n\nEsc или Enter — назад в меню");
        return text;
    }
    let mut text = String::from("NEON FRONTIER\n\nOPEN-WORLD ACTION • WINDOWS 11\n\n");
    for (index, option) in MENU_OPTIONS.iter().enumerate() {
        text.push_str(if index == selected { "> " } else { "  " });
        text.push_str(option_label(*option));
        text.push('\n');
    }
    text.push_str("\nСтрелки вверх / вниз — выбор • Enter — подтвердить\n");
    text.push_str(IN_GAME_HELP);
    text.push('\n');
    if !notice.is_empty() {
        text.push_str(notice);
        text.push('\n');
    }
    text.push_str(SHORT_CREDITS);
    text
}

fn wanted_status(state: &str) -> String {
    let Some((mode, stars)) = state.split_once(" • розыск: ") else {
        return state.to_string();
    };
    let level = stars.chars().filter(|character| *character == '★').count();
    let response = match level {
        0 => "патрули не разыскивают игрока",
        1 => "патруль начинает автомобильное преследование",
        2 => "патруль высаживает офицеров",
        3..=5 => "скорость автомобильного преследования растёт",
        _ => "максимальный уровень преследования",
    };
    format!("{mode}\nРОЗЫСК {level}/6 • {response}")
}

fn wrap_hud_line(line: &str) -> String {
    let mut wrapped = String::new();
    let mut current_chars = 0;
    for word in line.split_whitespace() {
        let word_chars = word.chars().count();
        if current_chars > 0 && current_chars + 1 + word_chars > HUD_MAX_LINE_CHARS {
            wrapped.push('\n');
            current_chars = 0;
        } else if current_chars > 0 {
            wrapped.push(' ');
            current_chars += 1;
        }
        wrapped.push_str(word);
        current_chars += word_chars;
    }
    wrapped
}

fn wrap_hud_text(text: &str) -> String {
    text.split('\n')
        .map(wrap_hud_line)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn health_line(health: f32) -> String {
    let clamped = health.clamp(0.0, 100.0);
    let rounded = clamped.round() as i32;
    if clamped <= 0.0 {
        format!("Здоровье: {rounded}% • нужен новый заезд")
    } else if clamped <= 25.0 {
        format!("Здоровье: {rounded}% • тяжело ранен")
    } else {
        format!("Здоровье: {rounded}%")
    }
}

pub fn compose_hud(
    objective: &str,
    credits: u32,
    health: f32,
    state: &str,
    interaction: &str,
    speed: &str,
    mouse_hint: &str,
    notice: &str,
) -> String {
    let objective = if objective.contains("Маршрут выполнен") {
        ""
    } else {
        objective
    };
    let notice = if notice.contains("Маршрут выполнен") {
        ""
    } else {
        notice
    };
    let hud = format!(
        "NEON FRONTIER • ранняя версия\n\n{}\nКредиты: {}\n{}\n\n{}\n{}\n{}\n\nWASD: движение/газ • Shift: бег\nSpace: прыжок • F: удар\n{}\nB: bloom • E: машина/выход • F5: сохранить • F9: загрузить\n{}",
        objective,
        credits,
        health_line(health),
        wanted_status(state),
        interaction,
        speed,
        mouse_hint,
        notice
    );
    wrap_hud_text(&hud)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_menu_lists_the_three_entries() {
        let menu = main_menu_text(0, false, "");
        assert!(menu.contains("Новая игра"));
        assert!(menu.contains("Загрузить сохранение"));
        assert!(menu.contains("Кредитс"));
        assert!(menu.contains(SHORT_CREDITS));
        assert!(menu.contains("Enter"));
    }

    #[test]
    fn menu_hit_testing_maps_each_row_and_ignores_other_areas() {
        assert_eq!(menu_index_at_cursor(Vec2::new(30.0, 151.0)), Some(0));
        assert_eq!(menu_index_at_cursor(Vec2::new(30.0, 175.0)), Some(1));
        assert_eq!(menu_index_at_cursor(Vec2::new(30.0, 199.0)), Some(2));
        assert_eq!(menu_index_at_cursor(Vec2::new(10.0, 151.0)), None);
        assert_eq!(menu_index_at_cursor(Vec2::new(30.0, 139.0)), None);
        assert_eq!(menu_index_at_cursor(Vec2::new(30.0, 212.0)), None);
        assert_eq!(menu_index_at_cursor(Vec2::new(f32::NAN, 151.0)), None);
    }

    #[test]
    fn selected_entry_is_marked_and_moves_with_the_cursor() {
        let first = main_menu_text(0, false, "");
        let second = main_menu_text(1, false, "");
        assert!(first.contains("> Новая игра"));
        assert!(!first.contains("> Загрузить сохранение"));
        assert!(second.contains("> Загрузить сохранение"));
        assert!(!second.contains("> Новая игра"));
    }

    #[test]
    fn credits_screen_names_the_authors_and_returns_hint() {
        let credits = main_menu_text(0, true, "");
        assert!(credits.contains("КРЕДИТС"));
        assert!(credits.contains(SHORT_CREDITS));
        assert!(credits.contains("назад в меню"));
        assert!(!credits.contains("> Новая игра"));
    }

    #[test]
    fn menu_shows_notice_after_a_failed_load() {
        let menu = main_menu_text(0, false, "Не удалось загрузить сохранение");
        assert!(menu.contains("Не удалось загрузить сохранение"));
    }

    #[test]
    fn health_line_reports_whole_percentages_and_damage_states() {
        assert_eq!(health_line(100.0), "Здоровье: 100%");
        assert_eq!(health_line(64.4), "Здоровье: 64%");
        assert!(health_line(20.0).contains("тяжело ранен"));
        assert!(health_line(0.0).contains("нужен новый заезд"));
        assert!(health_line(-5.0).contains("0%"));
        assert_eq!(health_line(f32::NAN).contains('%'), true);
    }

    #[test]
    fn hud_shows_objective_controls_and_notice() {
        let hud = compose_hud(
            "Маршрут «Три маяка»: точка 1/3 • 12 м",
            100,
            73.0,
            "ПЕШКОМ",
            "E: сесть в машину",
            "",
            "ЛКМ: захватить мышь и управлять камерой",
            "Сохранение загружено",
        );

        assert!(hud.contains("точка 1/3"));
        assert!(hud.contains("Кредиты: 100"));
        assert!(hud.contains("Здоровье: 73%"));
        assert!(hud.contains("WASD: движение/газ"));
        assert!(hud.contains("Space: прыжок • F: удар"));
        assert!(hud.contains("ПЕШКОМ"));
        assert!(hud.contains("Сохранение загружено"));
    }

    #[test]
    fn long_hud_lines_wrap_at_word_boundaries() {
        let hud = compose_hud(
            "Маршрут «Ночной маршрут»: контрольная точка северного района у набережной • 123 м",
            999,
            20.0,
            "ПАУЗА • розыск: ★★★★★★ • 23:59",
            "Сбавь скорость и нажми E, чтобы безопасно выйти из машины",
            "Скорость: 24.0 м/с",
            "ЛКМ: захватить мышь и управлять камерой",
            "Сохранение не удалось загрузить, файл повреждён или отсутствует",
        );

        assert!(hud.contains("контрольная точка"));
        assert!(hud.contains("файл повреждён или отсутствует"));
        assert!(hud
            .lines()
            .all(|line| line.chars().count() <= HUD_MAX_LINE_CHARS));
    }

    #[test]
    fn legacy_completed_route_state_is_not_rendered_or_persisted_in_hud() {
        let hud = compose_hud(
            "Маршрут выполнен! Награда получена",
            100,
            100.0,
            "ПЕШКОМ",
            "E: сесть в машину",
            "",
            "ЛКМ: захватить мышь и управлять камерой",
            "Маршрут выполнен! Награда получена",
        );

        assert!(!hud.contains("Маршрут выполнен"));
        assert!(!hud.contains("Маршрут продолжается"));
    }

    #[test]
    fn wanted_levels_are_numbered_out_of_six_and_show_distinct_pursuit() {
        let one_star = wanted_status("ПЕШКОМ • розыск: ★");
        let two_stars = wanted_status("ПЕШКОМ • розыск: ★★");
        let six_stars = wanted_status("ПЕШКОМ • розыск: ★★★★★★");

        assert!(one_star.contains("РОЗЫСК 1/6"));
        assert!(two_stars.contains("РОЗЫСК 2/6"));
        assert!(six_stars.contains("РОЗЫСК 6/6"));
        assert_ne!(one_star, two_stars);
    }
}
