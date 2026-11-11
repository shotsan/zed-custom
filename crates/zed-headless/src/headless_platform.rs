use gpui::{
    AnyWindowHandle, BackgroundExecutor, ClipboardItem, CursorStyle, DevicePixels,
    ForegroundExecutor, Keymap, Platform, PlatformDisplay, PlatformKeyboardLayout,
    PlatformKeyboardMapper, PlatformTextSystem, PromptButton, ScreenCaptureSource, Task,
    WindowAppearance, WindowParams,
};
use anyhow::Result;
use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};
use futures::channel::oneshot;

pub struct HeadlessPlatform {
    background_executor: BackgroundExecutor,
    foreground_executor: ForegroundExecutor,
}

impl HeadlessPlatform {
    pub fn new(
        background_executor: BackgroundExecutor,
        foreground_executor: ForegroundExecutor,
    ) -> Rc<Self> {
        Rc::new(Self {
            background_executor,
            foreground_executor,
        })
    }
}

struct DummyKeyboardLayout;
impl PlatformKeyboardLayout for DummyKeyboardLayout {
    fn id(&self) -> &str { "headless" }
    fn name(&self) -> &str { "headless" }
}

struct DummyKeyboardMapper;
impl PlatformKeyboardMapper for DummyKeyboardMapper {
    fn layout(&self) -> Box<dyn PlatformKeyboardLayout> { Box::new(DummyKeyboardLayout) }
    fn keystroke_from_sys_event(&self, _event: &gpui::PlatformInput) -> Option<gpui::Keystroke> { None }
    fn is_modifier_input(&self, _event: &gpui::PlatformInput) -> bool { false }
}

struct DummyTextSystem;
impl PlatformTextSystem for DummyTextSystem {
    fn add_fonts(&self, _fonts: Vec<Arc<Vec<u8>>>) -> gpui::Result<()> { Ok(()) }
    fn all_font_names(&self) -> Vec<String> { vec![] }
    fn all_system_fonts(&self) -> Vec<gpui::FontPath> { vec![] }
    fn font_id(&self, _descriptor: &gpui::Font) -> gpui::Result<gpui::FontId> { Ok(gpui::FontId(0)) }
    fn font_metrics(&self, _font_id: gpui::FontId) -> gpui::FontMetrics { gpui::FontMetrics { units_per_em: 1, ascent: 1.0, descent: 1.0, line_gap: 1.0, underline_position: 1.0, underline_thickness: 1.0, bounding_box: gpui::Bounds::default() } }
    fn typographic_bounds(&self, _font_id: gpui::FontId, _glyph_id: gpui::GlyphId) -> gpui::Result<gpui::Bounds<f32>> { Ok(gpui::Bounds::default()) }
    fn advance(&self, _font_id: gpui::FontId, _glyph_id: gpui::GlyphId) -> gpui::Result<gpui::Size<f32>> { Ok(gpui::Size::default()) }
    fn shape_line(&self, _text: &str, _font_size: f32, _runs: &[gpui::FontRun]) -> gpui::ShapedLine { gpui::ShapedLine { id: gpui::ShapedLineId(0), length: 0, text_width: 0.0, width: 0.0, render_runs: vec![], has_trailing_whitespace: false, trailing_whitespace_width: 0.0 } }
    fn rasterize_glyph(&self, _render_glyph: &gpui::RenderGlyph) -> gpui::Result<(gpui::Size<DevicePixels>, Vec<u8>)> { Ok((gpui::Size::default(), vec![])) }
    fn layout_line(&self, _text: &str, _font_size: f32, _runs: &[gpui::FontRun]) -> gpui::LayoutLine { gpui::LayoutLine { length: 0, width: 0.0, padding: 0.0, shape_runs: vec![], has_trailing_whitespace: false, trailing_whitespace_width: 0.0 } }
    fn wrap_line(&self, _text: &str, _font_id: gpui::FontId, _font_size: f32, _width: f32) -> Vec<usize> { vec![] }
}

impl Platform for HeadlessPlatform {
    fn background_executor(&self) -> BackgroundExecutor { self.background_executor.clone() }
    fn foreground_executor(&self) -> ForegroundExecutor { self.foreground_executor.clone() }
    fn text_system(&self) -> Arc<dyn PlatformTextSystem> { Arc::new(DummyTextSystem) }
    fn run(&self, on_finish_launching: Box<dyn FnOnce()>) { on_finish_launching(); }
    fn quit(&self) {}
    fn restart(&self, _path: Option<PathBuf>) {}
    fn activate(&self, _ignoring_other_apps: bool) {}
    fn hide(&self) {}
    fn hide_other_apps(&self) {}
    fn unhide_other_apps(&self) {}
    fn displays(&self) -> Vec<Rc<dyn PlatformDisplay>> { vec![] }
    fn primary_display(&self) -> Option<Rc<dyn PlatformDisplay>> { None }
    fn active_window(&self) -> Option<AnyWindowHandle> { None }
    fn open_window(&self, _handle: AnyWindowHandle, _params: WindowParams) -> Result<Box<dyn gpui::PlatformWindow>> { anyhow::bail!("Headless mode does not support windows") }
    fn window_appearance(&self) -> WindowAppearance { WindowAppearance::Light }
    fn open_url(&self, _url: &str) {}
    fn on_open_urls(&self, _callback: Box<dyn FnMut(Vec<String>)>) {}
    fn prompt_for_paths(&self, _options: gpui::PathPromptOptions) -> oneshot::Receiver<Result<Option<Vec<PathBuf>>>> { let (tx, rx) = oneshot::channel(); tx.send(Ok(None)).ok(); rx }
    fn prompt_for_new_path(&self, _directory: &Path, _suggested_name: Option<&str>) -> oneshot::Receiver<Result<Option<PathBuf>>> { let (tx, rx) = oneshot::channel(); tx.send(Ok(None)).ok(); rx }
    fn reveal_path(&self, _path: &Path) {}
    fn on_quit(&self, _callback: Box<dyn FnMut()>) {}
    fn on_reopen(&self, _callback: Box<dyn FnMut()>) {}
    fn set_menus(&self, _menus: Vec<gpui::Menu>, _keymap: &Keymap) {}
    fn set_dock_menu(&self, _menu: Vec<gpui::MenuItem>, _keymap: &Keymap) {}
    fn add_recent_document(&self, _path: &Path) {}
    fn on_app_menu_action(&self, _callback: Box<dyn FnMut(&dyn gpui::Action)>) {}
    fn on_will_open_app_menu(&self, _callback: Box<dyn FnMut()>) {}
    fn on_validate_app_menu_command(&self, _callback: Box<dyn FnMut(&dyn gpui::Action) -> bool>) {}
    fn app_path(&self) -> Result<PathBuf> { Ok(PathBuf::from("/usr/bin/zed-headless")) }
    fn path_for_auxiliary_executable(&self, _name: &str) -> Result<PathBuf> { Ok(PathBuf::from("/usr/bin/zed-headless")) }
    fn set_cursor_style(&self, _style: CursorStyle) {}
    fn should_auto_hide_scrollbars(&self) -> bool { false }
    fn read_from_clipboard(&self) -> Option<ClipboardItem> { None }
    fn write_to_clipboard(&self, _item: ClipboardItem) {}
    fn write_credentials(&self, _url: &str, _username: &str, _password: &[u8]) -> Task<Result<()>> { Task::ready(Ok(())) }
    fn read_credentials(&self, _url: &str) -> Task<Result<Option<(String, Vec<u8>)>>> { Task::ready(Ok(None)) }
    fn delete_credentials(&self, _url: &str) -> Task<Result<()>> { Task::ready(Ok(())) }
    fn register_url_scheme(&self, _scheme: &str) -> Task<Result<()>> { Task::ready(Ok(())) }
    fn open_with_system(&self, _path: &Path) {}
    fn keyboard_layout(&self) -> Box<dyn PlatformKeyboardLayout> { Box::new(DummyKeyboardLayout) }
    fn keyboard_mapper(&self) -> Rc<dyn PlatformKeyboardMapper> { Rc::new(DummyKeyboardMapper) }
    fn on_keyboard_layout_change(&self, _callback: Box<dyn FnMut()>) {}
    fn can_select_mixed_files_and_dirs(&self) -> bool { false }
}
