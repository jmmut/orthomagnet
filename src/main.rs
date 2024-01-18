use macroquad::prelude::*;

const SIZE: i32 = 16;

const DEFAULT_WINDOW_WIDTH: i32 = 800;
const DEFAULT_WINDOW_HEIGHT: i32 = 800;
const DEFAULT_WINDOW_TITLE: &str = "MY_CRATE_NAME";

const WHITE_HINT: Color = Color::new(1.0, 1.0, 1.0, 0.3);
const BLACK_HINT: Color = Color::new(0.0, 0.0, 0.0, 0.3);
const WHITE_FULL: Color = Color::new(1.0, 1.0, 1.0, 0.7);
const BLACK_FULL: Color = Color::new(0.0, 0.0, 0.0, 0.7);

#[macroquad::main(window_conf)]
async fn main() {
    let mut white_turn = true;
    let mut stones = Vec::new();
    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        clear_background(GRAY);
        let sw = screen_width();

        let board_rect = Rect::new(sw * 0.2, sw * 0.1, sw * 0.6, sw * 0.6);
        let tile_size_x = board_rect.w / SIZE as f32;
        let tile_size_y = board_rect.h / SIZE as f32;
        draw_board(board_rect);
        if let Some(tile) = get_tile(board_rect, SIZE, Vec2::from(mouse_position())) {
            let color = if white_turn { WHITE_HINT } else { BLACK_HINT };
            draw_rectangle(
                board_rect.x + tile.x as f32 * tile_size_x,
                board_rect.y + tile.y as f32 * tile_size_y,
                tile_size_x,
                tile_size_y,
                color,
            );
            if is_mouse_button_released(MouseButton::Left) {
                stones.push(tile);
                white_turn = !white_turn;
            }
        }
        // for stone in stones
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
    for vertical_line_i in 0..SIZE + 1 {
        let dx_i = vertical_line_i as f32 * dx;
        draw_line(x + dx_i, y, x + dx_i, y + h, 2.0, DARKGRAY);
    }

    for horizontal_line_i in 0..SIZE + 1 {
        let dy_i = horizontal_line_i as f32 * dy;
        draw_line(x, y + dy_i, x + w, y + dy_i, 2.0, DARKGRAY);
    }
}

fn get_tile(board_rect: Rect, size: i32, pos: Vec2) -> Option<IVec2> {
    if board_rect.contains(pos) {
        let tile_size_x = board_rect.w / size as f32;
        let tile_size_y = board_rect.h / size as f32;
        let x_i = ((pos.x - board_rect.x) / tile_size_x) as i32;
        let y_i = ((pos.y - board_rect.y) / tile_size_y) as i32;
        Some(IVec2::new(x_i, y_i))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tile_outside() {
        let tile = get_tile(Rect::new(20.0, 40.0, 100.0, 120.0), 8, Vec2::new(0.0, 0.0));
        assert_eq!(tile, None);
    }

    #[test]
    fn test_get_tile_inside() {
        let tile = get_tile(Rect::new(20.0, 40.0, 80.0, 80.0), 8, Vec2::new(45.0, 55.0));
        assert_eq!(tile, Some(IVec2::new(2, 1)));
    }
}
