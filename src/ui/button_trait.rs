use juquad::input::input_trait::InputTrait;
use juquad::widgets::button::{Interaction, Style};
use juquad::widgets::Widget;
use macroquad::input::MouseButton;

pub struct ButtonBase {
    pub interaction: Interaction,
    pub input: Box<dyn InputTrait>,
}
pub trait ButtonTrait: Widget {
    fn interaction(&self) -> Interaction;
    fn interaction_mut(&mut self) -> &mut Interaction;
    fn input(&self) -> &Box<dyn InputTrait>;
    fn interact(&mut self) -> Interaction {
        let input = self.input();
        let interaction = if self.rect().contains(input.mouse_position()) {
            if input.is_mouse_button_down(MouseButton::Left) {
                Interaction::Pressing
            } else if input.is_mouse_button_released(MouseButton::Left) {
                Interaction::Clicked
            } else {
                Interaction::Hovered
            }
        } else {
            Interaction::None
        };
        *self.interaction_mut() = interaction;
        self.interaction()
    }
    fn render(&self, style: &Style);
}
