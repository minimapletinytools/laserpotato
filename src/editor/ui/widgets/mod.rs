pub mod button;
pub mod modal;
pub mod scrollbar;
pub mod text_input;

pub use button::*;
pub use modal::*;
pub use scrollbar::*;
pub use text_input::*;

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::KeyCode;

    #[test]
    fn scrollbar_calculation_test() {
        // 1. Full visible page (no scrolling needed)
        let (thumb_h, thumb_top) = scrollbar::calculate_thumb_layout(10, 10, 0, 200.0);
        assert_eq!(thumb_h, 200.0);
        assert_eq!(thumb_top, 0.0);

        // 2. Proportional scrolling with 50 items, 10 visible, offset 0
        let (thumb_h, thumb_top) = scrollbar::calculate_thumb_layout(10, 50, 0, 200.0);
        assert_eq!(thumb_h, 40.0); // 10/50 * 200 = 40.0
        assert_eq!(thumb_top, 0.0);

        // 3. Offset at end (40 / 40)
        let (thumb_h, thumb_top) = scrollbar::calculate_thumb_layout(10, 50, 40, 200.0);
        assert_eq!(thumb_h, 40.0);
        assert_eq!(thumb_top, 160.0); // 200 - 40 = 160.0

        // 4. Drag scroll calculation
        let new_offset = scrollbar::calculate_drag_scroll_offset(0, 80.0, 200.0, 10, 50);
        assert_eq!(new_offset, 20); // 80 / 160 * 40 = 20

        // 5. Direct click jump
        let jump_offset = scrollbar::calculate_click_jump_offset(0.5, 10, 50);
        assert_eq!(jump_offset, 20); // 0.5 * 40 = 20
    }

    #[test]
    fn text_input_handling_test() {
        let mut buffer = String::new();

        // 1. Type characters
        assert!(text_input::handle_text_input_key(&mut buffer, KeyCode::KeyH, true, None));
        assert!(text_input::handle_text_input_key(&mut buffer, KeyCode::KeyE, false, None));
        assert!(text_input::handle_text_input_key(&mut buffer, KeyCode::KeyL, false, None));
        assert!(text_input::handle_text_input_key(&mut buffer, KeyCode::KeyL, false, None));
        assert!(text_input::handle_text_input_key(&mut buffer, KeyCode::KeyO, false, None));
        assert_eq!(buffer, "Hello");

        // 2. Backspace
        assert!(text_input::handle_text_input_key(&mut buffer, KeyCode::Backspace, false, None));
        assert_eq!(buffer, "Hell");

        // 3. Max length limit
        assert!(text_input::handle_text_input_key(&mut buffer, KeyCode::KeyO, false, Some(5)));
        assert_eq!(buffer, "Hello");
        assert!(!text_input::handle_text_input_key(&mut buffer, KeyCode::KeyW, false, Some(5)));
        assert_eq!(buffer, "Hello");

        // 4. Cursor formatting
        assert_eq!(text_input::format_input_with_cursor(&buffer, true), "Hello_");
        assert_eq!(text_input::format_input_with_cursor(&buffer, false), "Hello");
    }
}
