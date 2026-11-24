use bevy::prelude::{Component, FontSize, FontSource, TextFont};

pub const MAIN_MENU_TEXT: &str = "NEON FRONTIER\n\nOPEN-WORLD ACTION • WINDOWS 11\n\nНажми Enter или ЛКМ, чтобы начать\nWASD — движение / газ • Shift — бег\nSpace — прыжок • E — машина / выход\nB — Bloom • F5 — сохранить • F9 — загрузить\n\nСделано FoxVuk и Claude Opus 5.5";

#[derive(Component)]
pub struct Hud;

pub fn hud_font() -> TextFont {
    TextFont {
        font: FontSource::families("'Segoe UI', Arial, sans-serif"),
        font_size: FontSize::Px(19.0),
        ..Default::default()
    }
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
        "NEON FRONTIER • ранняя версия\n{}\nКредиты: {}\n{}\n{}\n{}\nWASD: движение/газ • Shift: бег • Space: прыжок\n{}\nB: bloom • F5: сохранить • F9: загрузить\n{}",
        objective, credits, state, interaction, speed, mouse_hint, notice
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
        assert!(hud.contains("Space: прыжок"));
        assert!(hud.contains("Сохранение загружено"));
    }
}
