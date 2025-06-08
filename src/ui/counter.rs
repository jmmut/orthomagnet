use crate::{new_button_from_text_rect, new_text_alt_font, FONT};
use juquad::draw::draw_rect;
use juquad::widgets::anchor::{Anchor, Horizontal};
use juquad::widgets::button::{Button, Style};
use juquad::widgets::button_group::LabelGroup;
use juquad::widgets::text::TextRect;
use macroquad::prelude::{Rect, DARKGRAY, LIGHTGRAY};

pub struct Counter {
    pub vertical_pad: f32,
    pub increase: Button,
    pub counter: TextRect,
    pub decrease: Button,
}

impl Counter {
    pub fn new(count: i32, position: Anchor, vertical_pad: f32, font_size: f32) -> Self {
        // let tmp_anchor = Anchor::top_left(0.0, 0.0);
        let group = LabelGroup::new_with_font(font_size, unsafe { FONT }, position);
        let [increase, counter, decrease] = group.create(["+", count.to_string().as_str(), "-"]);
        let increase = new_button_from_text_rect(increase);
        let decrease = new_button_from_text_rect(decrease);
        // let mut increase = new_button_alt_font("+", tmp_anchor, font_size);
        // let mut counter = TextRect::new(
        //     count.to_string().as_str(),
        //     from_below(increase.rect(), 0.0, vertical_pad),
        //     font_size,
        // );
        // let mut decrease = new_button_alt_font(
        //     "-",
        //     Anchor::from_below(counter.rect, 0.0, vertical_pad),
        //     font_size,
        // );
        // let mut rect = increase
        //     .rect()
        //     .combine_with(counter.rect)
        //     .combine_with(decrease.rect());
        //
        // let diff = position.get_top_left_pixel(rect.size());
        // increase.text_rect.rect = increase.text_rect.rect.offset(diff);
        // counter.rect = counter.rect.offset(diff);
        // decrease.text_rect.rect = decrease.text_rect.rect.offset(diff);
        // rect = rect.offset(diff);
        Self {
            vertical_pad,
            increase,
            counter,
            decrease,
        }
    }
    pub fn rect(&self) -> Rect {
        self.increase.rect().combine_with(self.decrease.rect())
    }

    pub fn update(&mut self, new_count: i32) {
        self.counter = new_text_alt_font(
            new_count.to_string().as_str(),
            Anchor::below(self.increase.rect(), Horizontal::Center, self.vertical_pad),
            self.counter.font_size,
        )
    }
    pub fn render(&self, style: &Style) {
        draw_rect(self.rect(), DARKGRAY);
        self.increase.render(style);
        self.counter.render_text(LIGHTGRAY);
        self.decrease.render(style);
    }
}
