// Tetris
use crate::tetris::gamestate::GameState;
use crate::tetris::tetlib::*;
use crate::vga_buffer::WRITER;

pub const WIDTH: usize = 10;
pub const HEIGHT: usize = 20;

pub fn run() {
    let mut grav_tick: usize = 250;

    let mut gs = GameState::new();
    let mut prev_scancode = 0; // required for key repeat

    // main loop
    loop {
        let prev_gs = gs.clone();

        // handle input
        let key = get_input(&mut prev_scancode);

        // quit
        if key == 'q' {
            break;
        }

        if key == 'p' {
            let mut key = get_input(&mut prev_scancode);
            put_text("P A U S E D");
            WRITER.lock().flush();
            while key != 'p' && key != 'q' {
                key = get_input(&mut prev_scancode);
                // unsafe { asm!("hlt") };
            }
        }

        // gravity
        if gs.counter >= grav_tick {
            if gravity(&mut gs.display, &mut gs.active_piece, &mut gs.next_piece) {
                gs.is_game_over = true;
                break;
            }
            gs.counter = 0;
        }

        // handle input
        handle_input(
            &mut gs.display,
            key,
            &mut gs.active_piece,
            &mut gs.next_piece,
            &mut grav_tick,
        );

        // hold piece
        if key == 'c' {
            hold(
                &mut gs.display,
                &mut gs.active_piece,
                &mut gs.hold_piece,
                &mut gs.next_piece,
            );
        }

        // full line
        full_line(&mut gs.display, &mut gs.gamescore, &mut grav_tick);

        // ghost piece
        ghost_piece(&mut gs.display, &mut gs.active_piece);

        // check if gs.display was changed
        let is_updated = gs != prev_gs || gs.is_game_over;

        // render
        render(
            &gs.display,
            is_updated,
            &mut gs.gamescore,
            &gs.hold_piece,
            &gs.next_piece,
            &grav_tick,
        );
        WRITER.lock().flush();
        gs.counter += 1;
    }
    put_text("G A M E  O V E R");
}
