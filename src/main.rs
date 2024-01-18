use macroquad::prelude::*;


const SIZE: i32 = 16;

const DEFAULT_WINDOW_WIDTH: i32 = 800;
const DEFAULT_WINDOW_HEIGHT: i32 = 800;
const DEFAULT_WINDOW_TITLE: &str = "MY_CRATE_NAME";

#[macroquad::main(window_conf)]
async fn main() {
    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        clear_background(GRAY);
        let sw = screen_width();

        draw_board(Rect::new(sw * 0.2, sw * 0.1, sw* 0.6, sw * 0.6));
        // draw_line(40.0, 40.0, 100.0, 200.0, 2.0, BLUE);
        // draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        // draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        //
        // draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}


fn window_conf() -> Conf {
    Conf {
        window_title: DEFAULT_WINDOW_TITLE.to_owned(),
        window_width: DEFAULT_WINDOW_WIDTH,
        window_height: DEFAULT_WINDOW_HEIGHT,
        high_dpi: true,
        ..Default::default()
    }
}

fn draw_board(rect: Rect) {
    let Rect { x, y, w, h } = rect;
    let dx = w / SIZE as f32;
    let dy = h / SIZE as f32;
    for vertical_line_i in 0..SIZE +1 {
        let dx_i = vertical_line_i as f32 * dx;
        draw_line(x + dx_i, y, x + dx_i, y + h, 2.0, DARKGRAY);
    }

    for horizontal_line_i in 0..SIZE +1 {
        let dy_i = horizontal_line_i as f32 * dy;
        draw_line(x, y + dy_i, x + w, y + dy_i, 2.0, DARKGRAY);
    }
}
