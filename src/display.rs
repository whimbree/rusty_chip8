const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Display {
    pub need_redraw: bool,
    pub fb: [bool; WIDTH*HEIGHT],
}

impl Display {
    pub fn new() -> Self {
        Display {
            need_redraw: false,
            fb: [false; WIDTH*HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        self.need_redraw = true;
        self.fb = [false; WIDTH*HEIGHT];
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, val: bool) {
        self.fb[x + y * WIDTH] = val;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        self.fb[x + y * WIDTH]
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let rows = sprite.len();
        let mut collision = false;
        for j in 0..rows {
            let row = sprite[j];
            for i in 0..8 {
                let new_value = row >> (7 - i) & 0x01;
                if new_value == 1 {
                    // Wraparound if goes out of bounds
                    let xi = (x + i) % 64;
                    let yj = (y + j) % 32;
                    let old_value = self.get_pixel(xi, yj);
                    if old_value {
                        collision = true;
                    }
                    self.set_pixel(xi, yj, (new_value == 1) ^ old_value);
                }
            }
        }
        self.need_redraw = true;
        collision
      }
}