//! Keyboard events.

use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;

pub use super::types::event::KeyEventInfo;

use log::info;

pub fn translate_event(keycode: &Keycode, keymod: &Mod) -> Option<KeyEventInfo> {
    /* Note: The analysis below encodes my US Mac Book Pro keyboard, almost completely. */
    /* Longer-term, we need a more complex design to handle other mappings and corresponding keyboard variations. */

    let shift = keymod.contains(sdl2::keyboard::Mod::LSHIFTMOD)
        || keymod.contains(sdl2::keyboard::Mod::RSHIFTMOD);
    let key = match &keycode {
        Keycode::Tab => "Tab".to_string(),
        Keycode::Space => " ".to_string(),
        Keycode::Return => "Enter".to_string(),
        Keycode::Left => "ArrowLeft".to_string(),
        Keycode::Right => "ArrowRight".to_string(),
        Keycode::Up => "ArrowUp".to_string(),
        Keycode::Down => "ArrowDown".to_string(),
        Keycode::Backspace => "Backspace".to_string(),
        Keycode::LShift => "Shift".to_string(),
        Keycode::Num0 => (if shift { ")" } else { "0" }).to_string(),
        Keycode::Num1 => (if shift { "!" } else { "1" }).to_string(),
        Keycode::Num2 => (if shift { "@" } else { "2" }).to_string(),
        Keycode::Num3 => (if shift { "#" } else { "3" }).to_string(),
        Keycode::Num4 => (if shift { "$" } else { "4" }).to_string(),
        Keycode::Num5 => (if shift { "%" } else { "5" }).to_string(),
        Keycode::Num6 => (if shift { "^" } else { "6" }).to_string(),
        Keycode::Num7 => (if shift { "&" } else { "7" }).to_string(),
        Keycode::Num8 => (if shift { "*" } else { "8" }).to_string(),
        Keycode::Num9 => (if shift { "(" } else { "9" }).to_string(),
        Keycode::A => (if shift { "A" } else { "a" }).to_string(),
        Keycode::B => (if shift { "B" } else { "b" }).to_string(),
        Keycode::C => (if shift { "C" } else { "c" }).to_string(),
        Keycode::D => (if shift { "D" } else { "d" }).to_string(),
        Keycode::E => (if shift { "E" } else { "e" }).to_string(),
        Keycode::F => (if shift { "F" } else { "f" }).to_string(),
        Keycode::G => (if shift { "G" } else { "g" }).to_string(),
        Keycode::H => (if shift { "H" } else { "h" }).to_string(),
        Keycode::I => (if shift { "I" } else { "i" }).to_string(),
        Keycode::J => (if shift { "J" } else { "j" }).to_string(),
        Keycode::K => (if shift { "K" } else { "k" }).to_string(),
        Keycode::L => (if shift { "L" } else { "l" }).to_string(),
        Keycode::M => (if shift { "M" } else { "m" }).to_string(),
        Keycode::N => (if shift { "N" } else { "n" }).to_string(),
        Keycode::O => (if shift { "O" } else { "o" }).to_string(),
        Keycode::P => (if shift { "P" } else { "p" }).to_string(),
        Keycode::Q => (if shift { "Q" } else { "q" }).to_string(),
        Keycode::R => (if shift { "R" } else { "r" }).to_string(),
        Keycode::S => (if shift { "S" } else { "s" }).to_string(),
        Keycode::T => (if shift { "T" } else { "t" }).to_string(),
        Keycode::U => (if shift { "U" } else { "u" }).to_string(),
        Keycode::V => (if shift { "V" } else { "v" }).to_string(),
        Keycode::W => (if shift { "W" } else { "w" }).to_string(),
        Keycode::X => (if shift { "X" } else { "x" }).to_string(),
        Keycode::Y => (if shift { "Y" } else { "y" }).to_string(),
        Keycode::Z => (if shift { "Z" } else { "z" }).to_string(),
        Keycode::Equals => (if shift { "+" } else { "=" }).to_string(),
        Keycode::Plus => "+".to_string(),
        Keycode::Slash => (if shift { "?" } else { "/" }).to_string(),
        Keycode::Question => "?".to_string(),
        Keycode::Period => (if shift { ">" } else { "." }).to_string(),
        Keycode::Greater => ">".to_string(),
        Keycode::Comma => (if shift { "<" } else { "," }).to_string(),
        Keycode::Less => "<".to_string(),
        Keycode::Backslash => (if shift { "|" } else { "\\" }).to_string(),
        Keycode::Colon => ":".to_string(),
        Keycode::Semicolon => (if shift { ":" } else { ";" }).to_string(),
        Keycode::At => "@".to_string(),
        Keycode::Minus => (if shift { "_" } else { "-" }).to_string(),
        Keycode::Underscore => "_".to_string(),
        Keycode::Exclaim => "!".to_string(),
        Keycode::Hash => "#".to_string(),
        Keycode::Backquote => (if shift { "~" } else { "`" }).to_string(),
        Keycode::Quote => (if shift { "\"" } else { "'" }).to_string(),
        Keycode::Quotedbl => "\"".to_string(),
        Keycode::LeftBracket => (if shift { "{" } else { "[" }).to_string(),
        Keycode::RightBracket => (if shift { "}" } else { "]" }).to_string(),

        /* More to consider later (but can we capture these in a browser?):
        Escape --- (Already caught, to quit.)
        CapsLock --- Remapped to Control, for me at least.
        F1--F12 --- Non-standard, but useful for experts' customization macros?
        Modifiers (??): LCtrl, LShift, LAlt, LGui, RCtrl, RShift, RAlt, RGui
         */
        keycode => {
            info!("Unrecognized key code, ignoring event: {:?}", keycode);
            return None;
        }
    };
    let event = KeyEventInfo {
        key: key,
        alt: keymod.contains(Mod::LALTMOD) || keymod.contains(Mod::RALTMOD),
        ctrl: keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD),
        meta: keymod.contains(Mod::LGUIMOD) || keymod.contains(Mod::RGUIMOD),
        shift: keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD),
    };
    Some(event)
}
