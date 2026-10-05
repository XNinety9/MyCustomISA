use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;

use slint::{Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, Timer, TimerMode, VecModel};

use crate::VirtualMachine;

slint::include_modules!();

// ── Key mapping ───────────────────────────────────────────────────────

// Slint Unicode values for special keys (from i-slint-common key_codes.rs)
const KEY_DELETE:    char = '\u{007F}';
const KEY_SHIFT:     char = '\u{0010}';
const KEY_CTRL:      char = '\u{0011}';
const KEY_ALT:       char = '\u{0012}';
const KEY_CAPS_LOCK: char = '\u{0014}';
const KEY_SHIFT_R:   char = '\u{0015}';
const KEY_CTRL_R:    char = '\u{0016}';
const KEY_UP:        char = '\u{F700}';
const KEY_DOWN:      char = '\u{F701}';
const KEY_LEFT:      char = '\u{F702}';
const KEY_RIGHT:     char = '\u{F703}';
// F1–F12: \u{F704}–\u{F70F}
const KEY_INSERT:    char = '\u{F727}';
const KEY_HOME:      char = '\u{F729}';
const KEY_END:       char = '\u{F72B}';
const KEY_PAGE_UP:   char = '\u{F72C}';
const KEY_PAGE_DOWN: char = '\u{F72D}';

fn is_modifier_key(c: char) -> bool {
    matches!(c, KEY_SHIFT | KEY_CTRL | KEY_ALT | KEY_CAPS_LOCK | KEY_SHIFT_R | KEY_CTRL_R)
}

fn slint_key_to_keycode(text: &SharedString) -> Option<u8> {
    let c = text.chars().next()?;
    if text.len() != c.len_utf8() { return None; } // multi-char strings not mapped

    match c {
        // Standard ASCII (0x00–0x7E), excluding 0x7F which is the Delete *key*
        c if (c as u32) < 0x7F => Some(c as u8),
        KEY_DELETE    => Some(0x99),
        KEY_UP        => Some(0x90),
        KEY_DOWN      => Some(0x91),
        KEY_LEFT      => Some(0x92),
        KEY_RIGHT     => Some(0x93),
        KEY_PAGE_UP   => Some(0x94),
        KEY_PAGE_DOWN => Some(0x95),
        KEY_HOME      => Some(0x96),
        KEY_END       => Some(0x97),
        KEY_INSERT    => Some(0x98),
        // F1–F12: 0xF704–0xF70F → 0x80–0x8B
        c if (c as u32) >= 0xF704 && (c as u32) <= 0xF70F =>
            Some(0x80 + (c as u32 - 0xF704) as u8),
        _ => None,
    }
}

// ── UI launch ─────────────────────────────────────────────────────────

pub fn launch(vm: VirtualMachine) {
    let vm = Rc::new(RefCell::new(vm));
    let app = AppWindow::new().expect("Failed to create window");

    sync_state(&app, &vm.borrow());

    // ── Load Binary ──────────────────────────────────────────────────
    {
        let vm_rc = vm.clone();
        let app_weak = app.as_weak();
        app.on_load_clicked(move || {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("Binary", &["bin"])
                .pick_file()
            else { return };

            {
                let mut state = vm_rc.borrow_mut();
                *state = VirtualMachine::new();
                if let Err(e) = state.load(path.to_str().unwrap_or("")) {
                    eprintln!("Load error: {e}");
                    return;
                }
            }

            if let Some(app) = app_weak.upgrade() {
                sync_state(&app, &vm_rc.borrow());
            }
        });
    }

    // ── Step ─────────────────────────────────────────────────────────
    {
        let vm_rc = vm.clone();
        let app_weak = app.as_weak();
        app.on_step_clicked(move || {
            vm_rc.borrow_mut().step();
            if let Some(app) = app_weak.upgrade() {
                sync_state(&app, &vm_rc.borrow());
            }
        });
    }

    // ── Run / Pause ───────────────────────────────────────────────────
    {
        let app_weak = app.as_weak();
        app.on_run_clicked(move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_is_running(true);
            }
        });
    }
    {
        let app_weak = app.as_weak();
        app.on_pause_clicked(move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_is_running(false);
            }
        });
    }

    // ── Keyboard: key-down ───────────────────────────────────────────
    {
        let vm_rc = vm.clone();
        let app_weak = app.as_weak();
        app.on_key_down(move |text, shift, ctrl, alt| {
            let c = match text.chars().next() { Some(c) => c, None => return };

            // Handle Caps Lock toggle
            if c == KEY_CAPS_LOCK {
                let mut state = vm_rc.borrow_mut();
                state.keyboard.caps_lock = !state.keyboard.caps_lock;
                drop(state);
                if let Some(app) = app_weak.upgrade() {
                    sync_state(&app, &vm_rc.borrow());
                }
                return;
            }

            // Skip bare modifier key presses
            if is_modifier_key(c) { return; }

            if let Some(keycode) = slint_key_to_keycode(&text) {
                let mut state = vm_rc.borrow_mut();
                let caps = state.keyboard.caps_lock;
                let mods: u8 = (shift as u8)
                    | ((ctrl as u8) << 1)
                    | ((alt  as u8) << 2)
                    | ((caps as u8) << 3);
                state.key_down(keycode, mods);
                drop(state);
                if let Some(app) = app_weak.upgrade() {
                    sync_state(&app, &vm_rc.borrow());
                }
            }
        });
    }

    // ── Keyboard: key-up ─────────────────────────────────────────────
    {
        let vm_rc = vm.clone();
        app.on_key_up(move |text| {
            if let Some(keycode) = slint_key_to_keycode(&text) {
                vm_rc.borrow_mut().key_up(keycode);
            }
        });
    }

    // ── Run timer: ~10 000 steps per 16 ms tick ───────────────────────
    let run_timer = Timer::default();
    {
        let vm_rc = vm.clone();
        let app_weak = app.as_weak();
        run_timer.start(TimerMode::Repeated, Duration::from_millis(16), move || {
            let Some(app) = app_weak.upgrade() else { return };
            if !app.get_is_running() { return; }

            let halted = {
                let mut state = vm_rc.borrow_mut();
                let mut hit_halt = false;
                for _ in 0..10_000 {
                    if !state.step() { hit_halt = true; break; }
                }
                hit_halt
            };

            if halted { app.set_is_running(false); }
            sync_state(&app, &vm_rc.borrow());
        });
    }

    app.run().expect("Event loop error");
}

// ── State sync ────────────────────────────────────────────────────────

fn sync_state(app: &AppWindow, vm: &VirtualMachine) {
    app.set_pc_text(fmt_hex(vm.cpu.pc).into());
    app.set_sp_text(fmt_hex(vm.cpu.sp).into());
    app.set_is_halted(vm.halted);

    let (hex, decoded) = format_current_instruction(vm);
    app.set_instr_hex(hex.into());
    app.set_instr_decoded(decoded.into());

    app.set_flag_z(vm.cpu.flags & (1 << crate::FLAG_Z) != 0);
    app.set_flag_c(vm.cpu.flags & (1 << crate::FLAG_C) != 0);
    app.set_flag_v(vm.cpu.flags & (1 << crate::FLAG_V) != 0);
    app.set_flag_n(vm.cpu.flags & (1 << crate::FLAG_N) != 0);
    app.set_flag_r(vm.cpu.flags & (1 << crate::FLAG_R) != 0);

    app.set_registers(build_register_model(vm));
    app.set_display_image(build_display_image(&vm.ram));

    // Keyboard status
    let m = vm.keyboard.modifiers;
    app.set_kb_queue(vm.keyboard.queue.len() as i32);
    app.set_kb_shift(m & 0x01 != 0);
    app.set_kb_ctrl(m & 0x02 != 0);
    app.set_kb_alt(m & 0x04 != 0);
    app.set_kb_caps(vm.keyboard.caps_lock);
    app.set_kb_last_key(format!("0x{:02X}", vm.keyboard.last_keycode).into());

    app.set_hex_rows(build_hex_model(vm));
}

// ── Helpers ───────────────────────────────────────────────────────────

fn fmt_hex(v: u16) -> String {
    format!("0x{:04X}", v)
}

fn format_current_instruction(vm: &VirtualMachine) -> (String, String) {
    let pc = vm.cpu.pc as usize;
    if pc + 1 >= vm.ram.len() {
        return ("????".into(), "???".into());
    }
    let word = ((vm.ram[pc] as u16) << 8) | vm.ram[pc + 1] as u16;
    let opcode = (word & 0xF800) >> 11;
    let mode   = (word & 0x0600) >> 9;
    let is_32bit = opcode != 0 && (mode == 0b00 || mode == 0b11);
    let hex = if is_32bit && pc + 3 < vm.ram.len() {
        let next = ((vm.ram[pc + 2] as u16) << 8) | vm.ram[pc + 3] as u16;
        format!("0x{:04X} 0x{:04X}", word, next)
    } else {
        format!("0x{:04X}", word)
    };
    (hex, crate::decode_instruction(vm.cpu.pc, &vm.ram))
}

fn build_register_model(vm: &VirtualMachine) -> ModelRc<RegPair> {
    // 8 rows R0/R8 … R7/R15, then SP/PC
    let mut v: Vec<RegPair> = (0..8_usize)
        .map(|i| RegPair {
            left_name:   format!("R{}", i).into(),
            left_value:  fmt_hex(vm.cpu.registers[i]).into(),
            right_name:  format!("R{}", i + 8).into(),
            right_value: fmt_hex(vm.cpu.registers[i + 8]).into(),
        })
        .collect();
    v.push(RegPair {
        left_name:   "SP".into(),
        left_value:  fmt_hex(vm.cpu.sp).into(),
        right_name:  "PC".into(),
        right_value: fmt_hex(vm.cpu.pc).into(),
    });
    ModelRc::new(VecModel::from(v))
}

fn build_display_image(ram: &[u8]) -> Image {
    const FB: usize = 0xF500;
    const W: u32 = 128;
    const H: u32 = 128;

    let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(W, H);
    let pixels = buf.make_mut_slice();

    for y in 0..H as usize {
        for x in 0..W as usize {
            let idx = y * W as usize + x;
            let byte = FB + idx / 8;
            let on = byte < ram.len() && (ram[byte] >> (7 - idx % 8)) & 1 == 1;
            pixels[idx] = if on {
                Rgba8Pixel { r: 255, g: 255, b: 255, a: 255 }
            } else {
                Rgba8Pixel { r: 0, g: 0, b: 0, a: 255 }
            };
        }
    }

    Image::from_rgba8(buf)
}

fn build_hex_model(vm: &VirtualMachine) -> ModelRc<HexRow> {
    let (instr_start, instr_end) = vm.instruction_byte_range();
    let pc_row = (vm.cpu.pc / 16) as i32;
    let start_row = (pc_row - 8).max(0) as u16;
    let total_rows: u16 = 32;

    let rows: Vec<HexRow> = (0..total_rows).map(|i| {
        let row_addr = (start_row + i) * 16;
        let row_end  = row_addr + 15;

        // Determine which byte offsets within this row are highlighted
        let hi_start = if instr_start >= row_addr && instr_start <= row_end {
            Some((instr_start - row_addr) as usize)
        } else if instr_start < row_addr && instr_end >= row_addr {
            Some(0)
        } else {
            None
        };
        let hi_end = hi_start.map(|_| {
            let end = instr_end.min(row_end);
            (end - row_addr) as usize
        });
        let has_highlight = hi_start.is_some();

        // Build hex segments and ascii
        let mut pre_hex  = String::new();
        let mut hi_hex   = String::new();
        let mut post_hex = String::new();
        let mut ascii    = String::new();

        for j in 0..16usize {
            let addr = row_addr as usize + j;
            let byte = vm.ram[addr];
            let sep = if j == 8 { "  " } else { "" };

            let hex_part = format!("{}{:02X} ", sep, byte);

            match (hi_start, hi_end) {
                (Some(hs), Some(he)) if j >= hs && j <= he => hi_hex.push_str(&hex_part),
                (Some(hs), _)        if j < hs             => pre_hex.push_str(&hex_part),
                _                                           => post_hex.push_str(&hex_part),
            }

            ascii.push(if byte >= 0x20 && byte < 0x7F { byte as char } else { '.' });
        }

        HexRow {
            addr:          format!("0x{:04X}  ", row_addr).into(),
            pre_hex:       SharedString::from(pre_hex.trim_end()),
            hi_hex:        SharedString::from(hi_hex.trim_end()),
            post_hex:      SharedString::from(post_hex.trim_end()),
            ascii:         ascii.into(),
            has_highlight: has_highlight,
        }
    }).collect();

    ModelRc::new(VecModel::from(rows))
}
