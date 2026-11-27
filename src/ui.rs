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
const HUD_METER_WIDTH: usize = 10;
const WANTED_LEVELS: usize = 6;
const MISSION_CYCLE_CREDITS: u32 = 300;

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
        MenuOption::Credits => "Титры",
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
        let mut text = String::from("NEON FRONTIER\n\nТИТРЫ\n\n");
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
    let level = level.min(WANTED_LEVELS);
    let meter: String = (0..WANTED_LEVELS)
        .map(|index| if index < level { '★' } else { '☆' })
        .collect();
    let response = match level {
        0 => "патрули не разыскивают игрока",
        1 => "автомобильный патруль начал погоню",
        2 => "к погоне подключился пеший офицер",
        3 => "офицер ускорился, патрули меняют тактику",
        4 => "два офицера и усиленный автомобильный ответ",
        5 => "офицеры ведут частую стрельбу по прямой видимости",
        _ => "максимальная скорость и огневая мощь полиции",
    };
    format!("{mode}\nРОЗЫСК {level}/6 [{meter}] • {response}")
}

fn mission_progress_line(credits: u32) -> String {
    let progress = credits % MISSION_CYCLE_CREDITS;
    let filled = ((progress * HUD_METER_WIDTH as u32 + MISSION_CYCLE_CREDITS / 2)
        / MISSION_CYCLE_CREDITS) as usize;
    let meter = format!(
        "{}{}",
        "█".repeat(filled.min(HUD_METER_WIDTH)),
        "░".repeat(HUD_METER_WIDTH - filled.min(HUD_METER_WIDTH)),
    );
    let remaining = MISSION_CYCLE_CREDITS - progress;
    format!("До смены маршрута: [{meter}] {remaining} кр.")
}

fn wrap_hud_line(line: &str) -> String {
    let mut wrapped = String::new();
    let mut current_chars = 0;
    for word in line.split_whitespace() {
        let word_chars = word.chars().count();
        if current_chars > 0 && current_chars + 1 + word_chars <= HUD_MAX_LINE_CHARS {
            wrapped.push(' ');
            wrapped.push_str(word);
            current_chars += 1 + word_chars;
            continue;
        }
        if current_chars > 0 {
            wrapped.push('\n');
            current_chars = 0;
        }
        for character in word.chars() {
            if current_chars == HUD_MAX_LINE_CHARS {
                wrapped.push('\n');
                current_chars = 0;
            }
            wrapped.push(character);
            current_chars += 1;
        }
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
    let filled = ((rounded.clamp(0, 100) as usize * HUD_METER_WIDTH + 50) / 100)
        .min(HUD_METER_WIDTH);
    let meter = format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(HUD_METER_WIDTH - filled),
    );
    let status = if clamped <= 0.0 {
        " • нужен новый заезд"
    } else if clamped <= 25.0 {
        " • тяжело ранен"
    } else {
        ""
    };
    format!("Здоровье: [{meter}] {rounded}%{status}")
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
        "NEON FRONTIER • ранняя версия\n\n{}\nКредиты: {}\n{}\n{}\n\n{}\n{}\n{}\n\nWASD: движение/газ • Shift: бег\nSpace: прыжок • F: удар\n{}\nB: bloom • E: машина/выход • F5: сохранить • F9: загрузить\n{}",
        objective,
        credits,
        mission_progress_line(credits),
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
        assert!(menu.contains("Титры"));
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
        assert!(credits.contains("ТИТРЫ"));
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
    fn health_line_reports_a_visual_meter_and_damage_states() {
        assert_eq!(health_line(100.0), "Здоровье: [██████████] 100%");
        assert_eq!(health_line(64.4), "Здоровье: [██████░░░░] 64%");
        assert!(health_line(20.0).contains("тяжело ранен"));
        assert!(health_line(0.0).contains("нужен новый заезд"));
        assert!(health_line(-5.0).contains("0%"));
        assert!(health_line(f32::NAN).contains('%'));
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
        assert!(hud.contains("До смены маршрута: [███░░░░░░░] 200 кр."));
        assert!(hud.contains("Здоровье: [███████░░░] 73%"));
        assert!(hud.contains("WASD: движение/газ"));
        assert!(hud.contains("Space: прыжок • F: удар"));
        assert!(hud.contains("ПЕШКОМ"));
        assert!(hud.contains("Сохранение загружено"));
    }

    #[test]
    fn mission_route_meter_resets_at_each_three_hundred_credit_cycle() {
        assert!(mission_progress_line(0).contains("[░░░░░░░░░░] 300 кр."));
        assert!(mission_progress_line(100).contains("[███░░░░░░░] 200 кр."));
        assert!(mission_progress_line(200).contains("[███████░░░] 100 кр."));
        assert!(mission_progress_line(299).contains("[██████████] 1 кр."));
        assert!(mission_progress_line(300).contains("[░░░░░░░░░░] 300 кр."));
    }

    #[test]
    fn long_hud_lines_wrap_without_cutting_long_words() {
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
        assert!(hud.contains("файл повреждён"));
        assert!(hud.contains("отсутствует"));
        assert!(hud.lines().all(|line| line.chars().count() <= HUD_MAX_LINE_CHARS));
    }

    #[test]
    fn long_hud_words_are_split_without_losing_characters() {
        let word = "Щ".repeat(HUD_MAX_LINE_CHARS * 2 + 7);
        let wrapped = wrap_hud_line(&word);
        let actual: String = wrapped.chars().filter(|character| *character != '\n').collect();

        assert_eq!(actual, word);
        assert!(wrapped.lines().all(|line| line.chars().count() <= HUD_MAX_LINE_CHARS));
    }

    #[test]
    fn exact_limit_hud_word_does_not_add_an_empty_line() {
        let word = "Ж".repeat(HUD_MAX_LINE_CHARS);
        assert_eq!(wrap_hud_line(&word), word);
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
    fn wanted_levels_show_a_fixed_six_star_meter_and_distinct_pursuit() {
        let expected = [
            "патрули не разыскивают игрока",
            "автомобильный патруль начал погоню",
            "к погоне подключился пеший офицер",
            "офицер ускорился, патрули меняют тактику",
            "два офицера и усиленный автомобильный ответ",
            "офицеры ведут частую стрельбу по прямой видимости",
            "максимальная скорость и огневая мощь полиции",
        ];
        for (level, response) in expected.iter().enumerate() {
            let stars = "★".repeat(level);
            let status = wanted_status(&format!("ПЕШКОМ • розыск: {stars}"));
            assert!(status.contains(&format!("РОЗЫСК {level}/6")));
            assert!(status.contains(response));
        }

        assert!(wanted_status("ПЕШКОМ • розыск: ★").contains("[★☆☆☆☆☆]"));
        assert!(wanted_status("ПЕШКОМ • розыск: ★★").contains("[★★☆☆☆☆]"));
        assert!(wanted_status("ПЕШКОМ • розыск: ★★★★★★").contains("[★★★★★★]"));
    }
}
