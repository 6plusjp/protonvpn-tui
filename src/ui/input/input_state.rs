use crate::state::AppState;
use crate::ui::render::View;

pub struct InputState<'a> {
    pub state: &'a mut AppState,
    pub current_view: &'a mut View,
    pub pending_g: &'a mut bool,
    pub filter_mode: &'a mut bool,
    pub filter_input: &'a mut String,
}

impl<'a> InputState<'a> {
    pub fn new(
        state: &'a mut AppState,
        current_view: &'a mut View,
        pending_g: &'a mut bool,
        filter_mode: &'a mut bool,
        filter_input: &'a mut String,
    ) -> Self {
        Self {
            state,
            current_view,
            pending_g,
            filter_mode,
            filter_input,
        }
    }
}
