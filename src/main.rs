#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_os = "windows"))]
compile_error!("KeyDi is Windows-only.");

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use eframe::egui::{self, Color32, FontId, RichText, Vec2, ViewportCommand};

use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    HC_ACTION, HHOOK, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static BLOCKING: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code == HC_ACTION as i32 {
        let msg_id = w_param.0 as u32;
        let is_key = matches!(msg_id, WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP);
        if is_key && BLOCKING.load(Ordering::Relaxed) {
            return LRESULT(1);
        }
    }
    CallNextHookEx(HHOOK::default(), n_code, w_param, l_param)
}

fn spawn_hook_thread() -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("keydi-hook".into())
        .spawn(|| unsafe {
            let hmod: HINSTANCE = GetModuleHandleW(None)
                .expect("GetModuleHandleW failed")
                .into();
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), hmod, 0)
                .expect("SetWindowsHookExW failed");
            let mut msg = MSG::default();
            loop {
                let ret = GetMessageW(&mut msg, None, 0, 0);
                if ret.0 <= 0 { break; }
            }
            let _ = UnhookWindowsHookEx(hook);
        })
        .expect("failed to spawn hook thread")
}

struct KeyDiApp {
    keyboard_disabled: bool,
    _hook_thread: thread::JoinHandle<()>,
}

impl KeyDiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let hook_thread = spawn_hook_thread();
        std::panic::set_hook(Box::new(|_| {
            BLOCKING.store(false, Ordering::SeqCst);
        }));
        Self { keyboard_disabled: false, _hook_thread: hook_thread }
    }

    fn set_blocking(&mut self, block: bool) {
        BLOCKING.store(block, Ordering::SeqCst);
        self.keyboard_disabled = block;
    }
}

impl eframe::App for KeyDiApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        BLOCKING.store(false, Ordering::SeqCst);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Redraw only on user interaction, not continuously at 60 FPS.
        // This is the main RAM/CPU fix — egui idles at ~0% CPU when inactive.
        ctx.request_repaint_after(std::time::Duration::from_millis(500));

        self.keyboard_disabled = BLOCKING.load(Ordering::Relaxed);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(14.0);

                ui.label(
                    RichText::new("KeyDi")
                        .font(FontId::proportional(28.0))
                        .strong(),
                );

                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    let btn_w = 100.0;
                    let gap = 12.0;
                    let avail = ui.available_width();
                    ui.add_space((avail - btn_w * 2.0 - gap).max(0.0) / 2.0);

                    let disable_btn = egui::Button::new(
                        RichText::new("Disable").font(FontId::proportional(15.0)),
                    )
                    .min_size(Vec2::new(btn_w, 34.0))
                    .fill(if self.keyboard_disabled {
                        Color32::from_rgb(180, 60, 60)
                    } else {
                        Color32::from_rgb(220, 80, 80)
                    });
                    if ui.add(disable_btn).clicked() {
                        self.set_blocking(true);
                    }

                    ui.add_space(gap);

                    let enable_btn = egui::Button::new(
                        RichText::new("Enable").font(FontId::proportional(15.0)),
                    )
                    .min_size(Vec2::new(btn_w, 34.0))
                    .fill(if !self.keyboard_disabled {
                        Color32::from_rgb(40, 150, 80)
                    } else {
                        Color32::from_rgb(60, 180, 100)
                    });
                    if ui.add(enable_btn).clicked() {
                        self.set_blocking(false);
                    }
                });

                ui.add_space(18.0);

                if self.keyboard_disabled {
                    ui.label(
                        RichText::new("● Keyboard disabled")
                            .font(FontId::proportional(14.0))
                            .color(Color32::from_rgb(230, 60, 60)),
                    );
                } else {
                    ui.label(
                        RichText::new("● Keyboard active")
                            .font(FontId::proportional(14.0))
                            .color(Color32::from_rgb(50, 200, 100)),
                    );
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("KeyDi")
            .with_inner_size([300.0, 160.0])
            .with_min_inner_size([300.0, 160.0])
            .with_max_inner_size([300.0, 160.0])
            .with_resizable(false),
        // Disable vsync — reduces GPU/driver overhead at idle
        vsync: false,
        ..Default::default()
    };

    eframe::run_native(
        "KeyDi",
        options,
        Box::new(|cc| Box::new(KeyDiApp::new(cc)) as Box<dyn eframe::App>),
    )
}
