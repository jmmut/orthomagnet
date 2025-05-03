use crate::new_button;
use juquad::draw::draw_rect;
use juquad::widgets::anchor::Anchor;
use juquad::widgets::button::{Button, Style};
use juquad::widgets::text::TextRect;
use macroquad::prelude::{Rect, DARKGRAY, LIGHTGRAY};

pub struct Counter {
    pub vertical_pad: f32,
    pub increase: Button,
    pub counter: TextRect,
    pub decrease: Button,
    pub rect: Rect,
}

impl Counter {
    pub fn new(count: i32, position: Anchor, vertical_pad: f32, font_size: f32) -> Self {
        let tmp_anchor = Anchor::top_left(0.0, 0.0);
        let mut increase = new_button("+", tmp_anchor, font_size);
        let mut counter = TextRect::new(
            count.to_string().as_str(),
            from_below(increase.rect(), 0.0, vertical_pad),
            font_size,
        );
        let mut decrease = new_button(
            "-",
            Anchor::from_below(counter.rect, 0.0, vertical_pad),
            font_size,
        );
        let mut rect = increase
            .rect()
            .combine_with(counter.rect)
            .combine_with(decrease.rect());

        let diff = position.get_top_left_pixel(rect.size());
        increase.text_rect.rect = increase.text_rect.rect.offset(diff);
        counter.rect = counter.rect.offset(diff);
        decrease.text_rect.rect = decrease.text_rect.rect.offset(diff);
        rect = rect.offset(diff);
        Self {
            vertical_pad,
            increase,
            counter,
            decrease,
            rect,
        }
    }

    pub fn update(&mut self, new_count: i32) {
        self.counter = TextRect::new(
            new_count.to_string().as_str(),
            from_below(self.increase.rect(), 0.0, self.vertical_pad),
            self.counter.font_size,
        )
    }
    pub fn render(&self, style: &Style) {
        draw_rect(
            self.increase.rect().combine_with(self.decrease.rect()),
            DARKGRAY,
        );
        self.increase.render(style);
        self.counter.render_text(LIGHTGRAY);
        self.decrease.render(style);
    }
}

fn from_below(other: Rect, x_diff: f32, y_diff: f32) -> Anchor {
    Anchor::top_left(other.x + x_diff, other.y + other.h + y_diff)
}
