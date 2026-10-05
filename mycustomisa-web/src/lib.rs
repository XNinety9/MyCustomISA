// WebAssembly build of the MyCustomISA virtual machine.
//
// Build:  cargo build --release --target wasm32-unknown-unknown
//
// The host page owns the loop: it writes a program into RAM, calls
// `vm_run` once per animation frame and paints the framebuffer straight
// out of linear memory (RAM is a 64 KB slice at `vm_ram_ptr()`).

use std::cell::RefCell;

use mycustomisa_core::{decode_instruction, VirtualMachine};

thread_local! {
    static VM: RefCell<VirtualMachine> = RefCell::new(VirtualMachine::new());
    static TEXT: RefCell<String> = RefCell::new(String::new());
}

fn with_vm<T>(f: impl FnOnce(&mut VirtualMachine) -> T) -> T {
    VM.with(|vm| f(&mut vm.borrow_mut()))
}

/// Power-on reset: fresh CPU and zeroed RAM. Invalidates `vm_ram_ptr()`.
#[unsafe(no_mangle)]
pub extern "C" fn vm_reset() {
    with_vm(|vm| *vm = VirtualMachine::new());
}

/// Address of the 64 KB RAM in wasm linear memory.
#[unsafe(no_mangle)]
pub extern "C" fn vm_ram_ptr() -> *mut u8 {
    with_vm(|vm| vm.ram.as_mut_ptr())
}

/// Execute up to `steps` instructions. Returns how many ran (fewer on HALT).
#[unsafe(no_mangle)]
pub extern "C" fn vm_run(steps: u32) -> u32 {
    with_vm(|vm| {
        for done in 0..steps {
            if !vm.step() {
                return done;
            }
        }
        steps
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn vm_halted() -> u32 {
    with_vm(|vm| vm.halted as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn vm_key_down(keycode: u32, modifiers: u32) {
    with_vm(|vm| vm.key_down(keycode as u8, modifiers as u8));
}

#[unsafe(no_mangle)]
pub extern "C" fn vm_key_up(keycode: u32) {
    with_vm(|vm| vm.key_up(keycode as u8));
}

#[unsafe(no_mangle)]
pub extern "C" fn vm_pc() -> u32 {
    with_vm(|vm| vm.cpu.pc as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn vm_sp() -> u32 {
    with_vm(|vm| vm.cpu.sp as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn vm_flags() -> u32 {
    with_vm(|vm| vm.cpu.flags as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn vm_reg(index: u32) -> u32 {
    with_vm(|vm| vm.cpu.registers[(index & 0xF) as usize] as u32)
}

/// Disassemble the instruction at PC. The UTF-8 text is at
/// `vm_text_ptr()`; the return value is its length in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn vm_disasm() -> u32 {
    let text = with_vm(|vm| decode_instruction(vm.cpu.pc, &vm.ram));
    TEXT.with(|t| {
        let mut t = t.borrow_mut();
        *t = text;
        t.len() as u32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn vm_text_ptr() -> *const u8 {
    TEXT.with(|t| t.borrow().as_ptr())
}
