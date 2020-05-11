use std::collections::HashMap;
use std::collections::HashSet;

use sdl2::keyboard::Keycode;

pub struct Keyboard {
    pub keymap: HashMap<Keycode, u8>,
    pub keys: HashSet<u8>,
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            keymap: Keyboard::gen_keymap(),
            keys:  HashSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.keys.clear();
    } 

    fn gen_keymap() -> HashMap<Keycode, u8> {
        let mut keymap = HashMap::new();
    
        keymap.insert(Keycode::Num1, 0);
        keymap.insert(Keycode::Num2, 1);
        keymap.insert(Keycode::Num3, 2);
        keymap.insert(Keycode::Num4, 3);
        keymap.insert(Keycode::Q, 4);
        keymap.insert(Keycode::W, 5);
        keymap.insert(Keycode::E, 6);
        keymap.insert(Keycode::R, 7);
        keymap.insert(Keycode::A, 8);
        keymap.insert(Keycode::S, 9);
        keymap.insert(Keycode::D, 10);
        keymap.insert(Keycode::F, 11);
        keymap.insert(Keycode::Z, 12);
        keymap.insert(Keycode::X, 13);
        keymap.insert(Keycode::C, 14);
        keymap.insert(Keycode::V, 15);
    
        keymap
    }

    pub fn update_keys(&mut self, keys_pressed: HashSet<Keycode>) {
        // Map over the keys, only considering chip8 keys
        self.keys.clear();

        for x in keys_pressed.iter() {
            if self.keymap.contains_key(x) {
                self.keys.insert(*self.keymap.get(x).unwrap());
            }
        }
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.keys.contains(&key)
    }
}

