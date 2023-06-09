use core::fmt::{self, Write};
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

struct DoubleBuffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    pub color_code: ColorCode,
    buffer: &'static mut Buffer,
    double_buffer: DoubleBuffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;
                self.double_buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code,
                };
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.double_buffer.chars[row][col];
                self.double_buffer.chars[row - 1][col] = character;
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.double_buffer.chars[row][col] = blank;
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                0xdb | 0xb0 => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn flush(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let screen_char = self.double_buffer.chars[row][col];
                self.buffer.chars[row][col].write(screen_char);
            }
        }
    }

    pub fn move_to(&mut self, col: usize, row: usize) -> Option<()> {
        if row >= BUFFER_HEIGHT || col >= BUFFER_WIDTH {
            return None;
        }

        self.row_position = row + 2; // +2 because I got no idea
        self.column_position = col;

        Some(())
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: BUFFER_HEIGHT - 1,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        double_buffer: DoubleBuffer {
            chars: [[ScreenChar {
                ascii_character: b' ',
                color_code: ColorCode::new(Color::White, Color::Black),
            }; BUFFER_WIDTH]; BUFFER_HEIGHT],
        },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {$crate::vga_buffer::_print(format_args!($($arg)*))};
}

#[macro_export]
macro_rules! println {
    () => {$crate::print!("\n"); WRITER.lock().flush()};
    ($($arg:tt)*) => {$crate::print!("{}\n", format_args!($($arg)*)); WRITER.lock().flush()}; // $crate means we don't need to import print! macro for println!
}

// directly print a byte to the screen
#[macro_export]
macro_rules! dprint {
    ($x:expr) => {
        WRITER.lock().write_byte($x)
    }; // $crate means we don't need to import print! macro for println!
}

// needs to be public so we can use it in other modules but private implementation detail
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut writer = WRITER.lock();
    writer.write_fmt(args).unwrap();
}

pub fn change_color(color: Color) {
    WRITER.lock().color_code = ColorCode::new(color, Color::Black);
}

pub fn clear_screen() {
    for _ in 0..BUFFER_HEIGHT {
        println!();
    }
}
