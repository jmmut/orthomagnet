use juquad::input::input_trait::InputTrait;
use juquad::widgets::{interact, Interaction, Style, Widget};

pub struct ButtonBase {
    pub interaction: Interaction,
    pub input: Box<dyn InputTrait>,
}
pub trait ButtonTrait: Widget {
    fn interaction(&self) -> Interaction;
    fn interaction_mut(&mut self) -> &mut Interaction;
    fn input(&self) -> &Box<dyn InputTrait>;
    fn interact(&mut self) -> Interaction {
        *self.interaction_mut() = interact(self.rect(), self.input());
        self.interaction()
    }
    fn render(&self, style: &Style);
}
