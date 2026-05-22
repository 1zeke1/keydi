Minimal Windows utility to toggle system-wide keyboard input on and off. Built in Rust using eframe/egui for the GUI and a WH_KEYBOARD_LL low-level hook via the Windows API — no drivers, no admin rights required.

The hook runs on a dedicated thread with its own Win32 message pump, keeping the UI fully responsive at all times. Keyboard state is managed via an atomic flag, so toggling is instant and thread-safe. On exit (or crash), the hook is automatically removed and keyboard input is always restored.

Binary size: ~1 MB (release build with LTO, panic=abort, opt-level=z).
