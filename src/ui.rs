use bevy::prelude::{Component, FontSize, FontSource, TextFont};

pub const MAIN_MENU_TEXT: &str = "NEON FRONTIER\n\nOPEN-WORLD ACTION • WINDOWS 11\n\nНажми Enter или ЛКМ, чтобы начать\nWASD — движение / газ • Shift — бег\nSpace — прыжок • E — машина / выход\nF — удар по пешеходам • B — Bloom\nF5 — сохранить • F9 — загрузить\n\nСделано FoxVuk и Claude Opus 5.5";

#[derive(Component)]
pub struct Hud;

pub fn hud_font() -> TextFont {
    TextFont {
        font: FontSource::families("'Segoe UI', Arial, sans-serif"),
        font_size: FontSize::Px(19.0),
        ..Default::default()
    }
}

fn wanted_status(state: &str) -> String {
    let Some((mode, stars)) = state.split_once(" • розыск: ") else {
        return state.to_string();
    };
    let level = stars.chars().filter(|character| *character == '★').count();
    let response = match level {
        0 => "патрули не разыскивают игрока",
        1 => "патруль преследует на автомобиле",
        2 => "патруль ускоряет автомобильный перехват",
        3 => "преследование становится интенсивнее",
        4 => "городской розыск усиливается",
        _ => "максимальный уровень розыска",
    };
    format!("{mode}\nРОЗЫСК {level}/5 • {response}")
}

pub fn compose_hud(
    objective: &str,
    credits: u32,
    state: &str,
    interaction: &str,
    speed: &str,
    mouse_hint: &str,
    notice: &str,
) -> String {
    format!(
        "NEON FRONTIER • ранняя версия\n\n{}\nКредиты: {}\n\n{}\n{}\n{}\n\nWASD: движение/газ • Shift: бег\nSpace: прыжок • F: удар\n{}\nB: bloom • F5: сохранить • F9: загрузить\n{}",
        objective,
        credits,
        wanted_status(state),
        interaction,
        speed,
        mouse_hint,
        notice
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_menu_shows_start_instructions_and_requested_credit() {
        assert!(MAIN_MENU_TEXT.contains("Нажми Enter"));
        assert!(MAIN_MENU_TEXT.contains("Сделано FoxVuk и Claude Opus 5.5"));
        assert!(MAIN_MENU_TEXT.contains("Space — прыжок"));
        assert!(MAIN_MENU_TEXT.contains("F — удар по пешеходам"));
    }

    #[test]
    fn hud_shows_objective_controls_and_notice() {
        let hud = compose_hud(
            "Маршрут «Три маяка»: точка 1/3 • 12 м",
            100,
            "ПЕШКОМ",
            "E: сесть в машину",
            "",
            "ЛКМ: захватить мышь и управлять камерой",
            "Сохранение загружено",
        );

        assert!(hud.contains("точка 1/3"));
        assert!(hud.contains("Кредиты: 100"));
        assert!(hud.contains("WASD: движение/газ"));
        assert!(hud.contains("Space: прыжок • F: удар"));
        assert!(hud.contains("ПЕШКОМ"));
        assert!(hud.contains("Сохранение загружено"));
    }

    #[test]
    fn first_and_second_wanted_stars_have_distinct_hud_statuses() {
        let one_star = wanted_status("ПЕШКОМ • розыск: ★");
        let two_stars = wanted_status("ПЕШКОМ • розыск: ★★");

        assert!(one_star.contains("РОЗЫСК 1/5 • патруль преследует на автомобиле"));
        assert!(two_stars.contains("РОЗЫСК 2/5 • патруль ускоряет автомобильный перехват"));
        assert_ne!(one_star, two_stars);
    }
}
