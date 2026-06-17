use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;

use slint::{Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, Timer, TimerMode, VecModel};

use crate::VirtualMachine;

slint::include_modules!();

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

    // Timer: runs ~10 000 steps per 16 ms tick (~60 Hz) while is_running.
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

            if halted {
                app.set_is_running(false);
            }
            sync_state(&app, &vm_rc.borrow());
        });
    }

    app.run().expect("Event loop error");
}

// ── helpers ──────────────────────────────────────────────────────────

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
}

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

fn build_register_model(vm: &VirtualMachine) -> ModelRc<RegEntry> {
    let mut v: Vec<RegEntry> = (0..16_usize)
        .map(|i| RegEntry {
            name:  format!("R{:<2}", i).into(),
            value: fmt_hex(vm.cpu.registers[i]).into(),
        })
        .collect();
    v.push(RegEntry { name: "SP".into(), value: fmt_hex(vm.cpu.sp).into() });
    v.push(RegEntry { name: "PC".into(), value: fmt_hex(vm.cpu.pc).into() });
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
