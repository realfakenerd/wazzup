use gpui::{div, px, rgb, Div, ParentElement, Styled};
use crate::theme::ThemeColors;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioPlayerState {
    pub file_path: Option<String>,
    pub is_playing: bool,
    pub progress_secs: u32,
    pub duration_secs: u32,
}

impl Default for AudioPlayerState {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPlayerState {
    pub fn new() -> Self {
        Self {
            file_path: None,
            is_playing: false,
            progress_secs: 0,
            duration_secs: 0,
        }
    }

    pub fn with_duration(file_path: Option<&str>, duration_secs: u32) -> Self {
        Self {
            file_path: file_path.map(|s| s.to_string()),
            is_playing: false,
            progress_secs: 0,
            duration_secs,
        }
    }

    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.progress_secs = 0;
    }

    pub fn seek(&mut self, to_secs: u32) {
        self.progress_secs = to_secs.min(self.duration_secs);
    }

    pub fn advance_progress(&mut self, delta_secs: u32) {
        self.progress_secs = (self.progress_secs + delta_secs).min(self.duration_secs);
        if self.progress_secs >= self.duration_secs {
            self.is_playing = false;
        }
    }

    pub fn progress_ratio(&self) -> f32 {
        if self.duration_secs == 0 {
            0.0
        } else {
            (self.progress_secs as f32 / self.duration_secs as f32).clamp(0.0, 1.0)
        }
    }
}

pub fn format_duration(seconds: u32) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", mins, secs)
}

pub fn render_audio_player(state: &AudioPlayerState) -> Div {
    let play_icon = if state.is_playing { "⏸" } else { "▶" };
    let time_str = format!(
        "{} / {}",
        format_duration(state.progress_secs),
        format_duration(state.duration_secs)
    );

    let progress_pct = (state.progress_ratio() * 100.0).round() as u32;

    div()
        .flex()
        .items_center()
        .gap_3()
        .px_3()
        .py_2()
        .w(px(260.0))
        .rounded_lg()
        .bg(rgb(ThemeColors::SURFACE))
        .border_1()
        .border_color(rgb(ThemeColors::BORDER))
        // Play/Pause circular button
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w_8()
                .h_8()
                .rounded_full()
                .bg(ThemeColors::emerald())
                .text_color(rgb(ThemeColors::CRUST))
                .text_size(px(14.0))
                .cursor_pointer()
                .child(play_icon),
        )
        // Progress bar and time labels
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap_1()
                // Track bar
                .child(
                    div()
                        .flex()
                        .w_full()
                        .h_1p5()
                        .rounded_full()
                        .bg(rgb(ThemeColors::CRUST))
                        .overflow_hidden()
                        .child(
                            div()
                                .h_full()
                                .w(gpui::relative(progress_pct as f32 / 100.0))
                                .bg(ThemeColors::emerald()),
                        ),
                )
                // Duration label
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .text_size(px(10.0))
                        .text_color(ThemeColors::text_muted())
                        .child(time_str),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[core::prelude::v1::test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "00:00");
        assert_eq!(format_duration(65), "01:05");
        assert_eq!(format_duration(3600), "60:00");
    }

    #[core::prelude::v1::test]
    fn test_audio_player_state_lifecycle() {
        let mut player = AudioPlayerState::with_duration(Some("/tmp/voice.ogg"), 30);
        assert!(!player.is_playing);
        assert_eq!(player.progress_secs, 0);
        assert_eq!(player.progress_ratio(), 0.0);

        player.toggle_play();
        assert!(player.is_playing);

        player.advance_progress(15);
        assert_eq!(player.progress_secs, 15);
        assert!((player.progress_ratio() - 0.5).abs() < f32::EPSILON);

        player.advance_progress(20);
        assert_eq!(player.progress_secs, 30);
        assert!(!player.is_playing); // finished automatically
        assert!((player.progress_ratio() - 1.0).abs() < f32::EPSILON);

        player.stop();
        assert_eq!(player.progress_secs, 0);
    }

    #[core::prelude::v1::test]
    fn test_audio_player_seek() {
        let mut player = AudioPlayerState::with_duration(None, 100);
        player.seek(50);
        assert_eq!(player.progress_secs, 50);

        player.seek(200); // capped at duration
        assert_eq!(player.progress_secs, 100);
    }
}
