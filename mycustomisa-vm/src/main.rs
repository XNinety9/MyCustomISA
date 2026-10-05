pub mod ui;

pub use mycustomisa_core::*;

// ── Entry point ───────────────────────────────────────────────────────

fn main() {
    ui::launch(VirtualMachine::new());
}
