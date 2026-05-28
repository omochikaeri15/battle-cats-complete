use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// STRICT NAMESPACE RULES: We only import from the public build portal.
use nyanko::animation::build::{self, Rig, Animation};
use nyanko::animation::build::GlowRenderer;
use nyanko::animation::find;

use core::animation::logic::constants::{IDX_NONE, IDX_MODEL, IDX_SPIRIT, IDX_WALK, IDX_IDLE, IDX_ATTACK, IDX_KB, IDX_BURROW, IDX_SURFACE};
use core::animation::export::state::{ExporterState, ExportMode};
use core::settings::logic::state::Settings;
use crate::features::animation::controls;
use crate::features::animation::controls::render_controls_overlay;
use crate::global::shared::DragGuard;

pub struct AnimViewer {
    pub zoom_level: f32,
    pub target_zoom_level: f32,
    pub pan_offset: egui::Vec2,
    pub current_anim: Option<Animation>,
    pub current_frame: f32,
    pub is_playing: bool,
    pub playback_speed: f32,
    pub loop_range: (Option<i32>, Option<i32>),
    pub range_str_cache: (String, String),
    pub single_frame_str: String,
    pub speed_str: String,
    pub hold_timer: f32,
    pub hold_dir: i8,
    pub loaded_anim_index: usize,
    pub last_anim_index: usize, // NEW: Tracks when the user clicks a different animation tab
    pub loaded_id: String,
    pub summoner_id: String,
    last_loaded_id: String,
    pub pending_initial_center: bool,

    // The massive state reduction: We just hold an opaque Rig token now!
    pub held_rig: Option<Arc<Rig>>,
    pub renderer: Arc<Mutex<Option<GlowRenderer>>>,

    pub cached_controls_width: f32,
    pub cached_grid_height: f32,
    pub is_expanded: bool,
    pub texture_version: u64,
    pub is_pointer_over_controls: bool,
    pub is_viewport_dragging: bool,
    pub is_selecting_export_region: bool,
    pub export_selection_start: Option<egui::Pos2>,
    pub export_state: ExporterState,
    pub has_scanned_showcase: bool,
    pub was_export_popup_open: bool,
}

impl Default for AnimViewer {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            target_zoom_level: 1.0,
            pan_offset: egui::vec2(0.0, 0.0),
            current_anim: None,
            current_frame: 0.0,
            is_playing: true,
            playback_speed: 1.0,
            loop_range: (None, None),
            range_str_cache: (String::new(), String::new()),
            single_frame_str: String::new(),
            speed_str: String::new(),
            hold_timer: 0.0,
            hold_dir: 0,
            loaded_anim_index: 0,
            last_anim_index: usize::MAX, // Initialize to impossible value to force first load
            loaded_id: String::new(),
            summoner_id: String::new(),
            last_loaded_id: "FORCE_INIT".to_string(),
            pending_initial_center: false,
            held_rig: None,
            renderer: Arc::new(Mutex::new(None)),
            cached_controls_width: 0.0,
            cached_grid_height: 55.0,
            is_expanded: false,
            texture_version: 0,
            is_pointer_over_controls: false,
            is_viewport_dragging: false,
            is_selecting_export_region: false,
            export_selection_start: None,
            export_state: ExporterState::default(),
            has_scanned_showcase: false,
            was_export_popup_open: false,
        }
    }
}

impl AnimViewer {
    fn update_export_state(&mut self, _settings: &Settings) {
        self.export_state.loop_supported = self.loaded_anim_index == IDX_WALK || self.loaded_anim_index == IDX_IDLE;

        if self.export_state.export_mode != ExportMode::Showcase {
            if let Some(anim) = &self.current_anim {
                self.export_state.max_frame = anim.max_frame;
                self.export_state.frame_start = 0;
                self.export_state.frame_end = anim.max_frame;
            } else {
                self.export_state.max_frame = 0;
                self.export_state.frame_start = 0;
                self.export_state.frame_end = 0;
            }
            self.export_state.frame_start_str.clear();
            self.export_state.frame_end_str.clear();
        }

        let type_string = match self.loaded_anim_index {
            IDX_WALK => "walk",
            IDX_IDLE => "idle",
            IDX_ATTACK => "attack",
            IDX_KB => "kb",
            IDX_BURROW => "burrow",
            IDX_SURFACE => "surface",
            IDX_SPIRIT => "spirit",
            IDX_MODEL => "model",
            _ => "anim",
        };

        let raw_id = if self.loaded_anim_index == IDX_SPIRIT {
            if self.summoner_id.is_empty() { &self.loaded_id } else { &self.summoner_id }
        } else { &self.loaded_id };

        let id_parts: Vec<&str> = raw_id.split('_').collect();
        let mut clean_id = id_parts[0].to_string();

        if id_parts.len() >= 2 {
            if id_parts[0].chars().all(char::is_numeric) {
                let form_number = match id_parts[1].chars().next() {
                    Some('f') => 1, Some('c') => 2, Some('s') => 3, Some('u') => 4, _ => 0
                };
                if form_number > 0 { clean_id = format!("{}-{}", id_parts[0], form_number); }
            }
        }

        self.export_state.name_prefix = format!("{}.{}", clean_id, type_string);
    }

    pub fn load_anim(&mut self, path: &Path, settings: &Settings) {
        if let Ok(anim_bytes) = std::fs::read(path) {
            if let Some(anim) = Animation::parse(&anim_bytes) {
                self.current_frame = 0.0;
                self.loop_range = (None, None);
                self.range_str_cache = (String::new(), String::new());
                self.single_frame_str = "0".to_string();

                self.current_anim = Some(anim);
                self.update_export_state(settings);
                return;
            }
        }

        self.current_anim = None;
        self.current_frame = 0.0;
        self.loop_range = (None, None);
        self.range_str_cache = (String::new(), String::new());
        self.single_frame_str = "0".to_string();
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        primary_id: &str,
        secondary_id: &str,
        available_anims: &[(usize, PathBuf)],
        primary_assets: Option<(PathBuf, PathBuf, PathBuf)>,
        secondary_assets: Option<(PathBuf, PathBuf, PathBuf, PathBuf)>,
        rig_sync: &mut Option<Arc<Rig>>,
        settings: &mut Settings,
        _drag_guard: &mut DragGuard,
    ) {
        let base_assets_available = primary_assets.is_some();
        let secondary_available = secondary_assets.is_some();

        let current_index = self.loaded_anim_index;
        let mut valid_index = current_index;

        let is_current_valid = if current_index == IDX_NONE {
            false
        } else if current_index == IDX_SPIRIT {
            secondary_available
        } else if current_index == IDX_MODEL {
            base_assets_available
        } else {
            base_assets_available && available_anims.iter().any(|(index, _)| *index == current_index)
        };

        if !is_current_valid {
            valid_index = IDX_NONE;

            if base_assets_available {
                let priority_list = [IDX_WALK, IDX_IDLE, IDX_ATTACK, IDX_KB, IDX_BURROW, IDX_SURFACE];
                for check_index in priority_list {
                    if available_anims.iter().any(|(index, _)| *index == check_index) {
                        valid_index = check_index;
                        break;
                    }
                }
            }

            if valid_index == IDX_NONE && secondary_available { valid_index = IDX_SPIRIT; }
            if valid_index == IDX_NONE && base_assets_available { valid_index = IDX_MODEL; }
        }

        if valid_index != current_index {
            self.loaded_anim_index = valid_index;
            if valid_index == IDX_NONE {
                self.current_anim = None;
                self.held_rig = None;
                *rig_sync = None;
            }
            if current_index == IDX_NONE && valid_index != IDX_NONE {
                self.loaded_id.clear();
            }
        }

        // NEW: Track if the user requested a different animation tab
        let mut anim_index_changed = false;
        if self.last_anim_index != valid_index {
            self.last_anim_index = valid_index;
            anim_index_changed = true;
        }

        let target_viewer_id = if self.loaded_anim_index == IDX_SPIRIT { secondary_id.to_string() } else { primary_id.to_string() };

        let is_stable = self.loaded_id == target_viewer_id;
        let is_first_launch = self.held_rig.is_none() && rig_sync.is_none();

        if valid_index == IDX_NONE && !is_stable {
            self.loaded_id = target_viewer_id.clone();
        }

        if is_stable {
            if let Some(rig) = rig_sync {
                self.held_rig = Some(rig.clone());
            }
        }

        // LOAD NEW RIG FROM DISK
        if (!is_stable || is_first_launch) && valid_index != IDX_NONE {
            let (resolved_png, resolved_cut, resolved_model, resolved_anim) = resolve_paths(valid_index, &primary_assets, &secondary_assets, available_anims);

            let mut load_success = false;
            if let (Some(png_path), Some(cut_path), Some(model_path)) = (resolved_png, resolved_cut, resolved_model) {
                if let (Ok(png_bytes), Ok(cut_bytes), Ok(model_bytes)) = (
                    std::fs::read(png_path), std::fs::read(cut_path), std::fs::read(model_path)
                ) {
                    if let Some(parsed_rig) = Rig::parse(&png_bytes, &cut_bytes, &model_bytes) {
                        let arc_rig = Arc::new(parsed_rig);
                        self.held_rig = Some(arc_rig.clone());
                        *rig_sync = Some(arc_rig);

                        self.loaded_id = target_viewer_id.clone();
                        self.pending_initial_center = true;
                        load_success = true;
                    }
                }
            }

            if !load_success {
                self.loaded_id = target_viewer_id.clone();
                self.held_rig = None;
            } else if let Some(animation_path) = resolved_anim {
                self.load_anim(animation_path, settings);
            } else {
                self.current_anim = None;
                self.update_export_state(settings); // Ensure max_frame zeroed
            }
        } else if anim_index_changed && is_stable && valid_index != IDX_NONE {
            // FIX: If the rig is stable but the user swapped animations, fetch the new animation bytes!
            let (_, _, _, resolved_anim) = resolve_paths(valid_index, &primary_assets, &secondary_assets, available_anims);
            if let Some(animation_path) = resolved_anim {
                self.load_anim(animation_path, settings);
            } else {
                self.current_anim = None;
                self.update_export_state(settings);
            }
        }

        // UI Layout
        if self.is_expanded {
            egui::Area::new("expanded_anim_viewer_area".into())
                .fixed_pos(egui::pos2(0.0, 0.0))
                .order(egui::Order::Middle)
                .show(ctx, |ui| {
                    let screen_rect = ctx.screen_rect();
                    egui::Frame::window(&ctx.style())
                        .inner_margin(0.0)
                        .shadow(egui::epaint::Shadow::NONE)
                        .show(ui, |ui| {
                            ui.set_min_size(screen_rect.size());
                            ui.set_max_size(screen_rect.size());
                            let (viewport_rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
                            ui.put(viewport_rect, |ui: &mut egui::Ui| {
                                self.draw_viewport(
                                    ui, viewport_rect, available_anims, base_assets_available,
                                    primary_id, secondary_id, &secondary_assets, settings
                                );
                                ui.allocate_rect(viewport_rect, egui::Sense::hover())
                            });
                        });
                });

            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(egui::RichText::new("Animation Expanded").size(16.0).weak());
                if ui.button("Restore View").clicked() {
                    self.is_expanded = false;
                }
            });
        } else {
            ui.vertical(|ui| {
                let (viewport_rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
                ui.put(viewport_rect, |ui: &mut egui::Ui| {
                    self.draw_viewport(
                        ui, viewport_rect, available_anims, base_assets_available,
                        primary_id, secondary_id, &secondary_assets, settings
                    );
                    ui.allocate_rect(viewport_rect, egui::Sense::hover())
                });
            });
        }
    }

    fn draw_viewport(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        available_anims: &[(usize, PathBuf)],
        base_assets_available: bool,
        primary_id: &str,
        secondary_id: &str,
        secondary_assets: &Option<(PathBuf, PathBuf, PathBuf, PathBuf)>,
        settings: &mut Settings,
    ) {
        let delta_time = ui.input(|input_state| input_state.stable_dt);
        let interpolation = settings.animation.interpolation;
        let _debug_show_info = settings.animation.debug_view;
        let centering_behavior = settings.animation.centering_behavior;
        let native_fps = settings.animation.native_fps;

        let _just_finished_selection = false;

        if !primary_id.is_empty() { self.summoner_id = primary_id.to_string(); }

        if self.loaded_id != self.last_loaded_id {
            self.last_loaded_id = self.loaded_id.clone();
            self.pending_initial_center = true;
            self.has_scanned_showcase = false;
        }

        let mut new_center: Option<(egui::Vec2, f32)> = None;
        let mut should_clear_pending = false;

        if let Some(rig) = &self.held_rig {
            if self.pending_initial_center {
                if centering_behavior == 0 {
                    if !rig.is_empty() {
                        // TODO: Update bounds.rs next step to use Rig instead of Model/Sheet!
                        /*
                        if let Some((offset, fit_zoom)) = bounds::calculate_initial_view(rig, self.current_anim.as_ref(), ui.available_size().x, ui.available_size().y, settings.animation.use_tight_bounds) {
                            new_center = Some((egui::vec2(offset.x, offset.y), fit_zoom));
                        }
                        */
                    }
                } else if centering_behavior == 1 {
                    new_center = Some((egui::Vec2::ZERO, self.target_zoom_level));
                } else { should_clear_pending = true; }
            }
        } else { should_clear_pending = true; }

        if let (Some(offset), Some(zoom)) = (new_center.map(|c| c.0), new_center.map(|c| c.1)) {
            self.pan_offset = offset;
            if centering_behavior == 0 { self.target_zoom_level = zoom; }
            self.pending_initial_center = false;
        } else if should_clear_pending { self.pending_initial_center = false; }

        let zoom_difference = self.target_zoom_level - self.zoom_level;
        if zoom_difference.abs() > 0.001 { self.zoom_level += zoom_difference * 15.0 * delta_time; } else { self.zoom_level = self.target_zoom_level; }

        if let Some(animation) = &self.current_anim {
            let loop_lcm_max = if self.loaded_anim_index <= 1 { find::loop_frames(animation) } else { Some(animation.max_frame) };
            let range_start = self.loop_range.0.unwrap_or(0);
            let (effective_max, is_infinite, has_user_override) = match (self.loop_range.1, loop_lcm_max) {
                (Some(user_override), _) => (user_override as f32, false, true),
                (None, Some(calculated_max)) => (calculated_max as f32, false, false),
                (None, None) => (0.0, true, false),
            };

            if self.hold_dir != 0 {
                self.hold_timer += delta_time;
                ui.ctx().request_repaint();
                if self.hold_timer > 0.2 {
                    let speed_factor = if self.playback_speed.abs() < 0.05 { 1.0 } else { self.playback_speed.abs() };
                    let frame_delta = self.hold_dir as f32 * delta_time * 30.0 * speed_factor;
                    let mut new_frame = self.current_frame + frame_delta;
                    if !is_infinite {
                        if new_frame > effective_max { new_frame = 0.0; } else if new_frame < 0.0 { new_frame = effective_max; }
                    } else { if new_frame < 0.0 { new_frame = 0.0; } }
                    self.current_frame = new_frame;
                }
            } else { self.hold_timer = 0.0; }

            if self.is_playing {
                if !is_infinite && effective_max < 1.0 && !has_user_override { self.current_frame = 0.0; } else {
                    self.current_frame += delta_time * 30.0 * self.playback_speed;
                    if !is_infinite && self.current_frame > effective_max { self.current_frame = range_start as f32; }
                }
                ui.ctx().request_repaint();
            }
        }

        let (rect_alloc, viewport_response) = ui.allocate_exact_size(rect.size(), egui::Sense::drag());
        let (hover_position, right_mouse_down, left_mouse_down) = ui.input(|input_state| (input_state.pointer.hover_pos(), input_state.pointer.secondary_down(), input_state.pointer.primary_down()));

        let block_input = self.is_pointer_over_controls || (self.is_selecting_export_region && right_mouse_down);
        if self.is_selecting_export_region && left_mouse_down && hover_position.is_some() {
            controls::handle_viewport_input(ui, &viewport_response, &mut self.pan_offset, &mut self.zoom_level, &mut self.target_zoom_level, &mut self.pending_initial_center, false, &mut self.is_viewport_dragging);
        } else {
            controls::handle_viewport_input(ui, &viewport_response, &mut self.pan_offset, &mut self.zoom_level, &mut self.target_zoom_level, &mut self.pending_initial_center, block_input, &mut self.is_viewport_dragging);
        }

        let _showcase_render_time = 0.0;

        if let Some(rig) = &self.held_rig {

            if self.export_state.is_processing {
                ui.ctx().request_repaint();
            }

            // FIX: We ALWAYS render if we have a rig!
            // If current_anim is None (user selected "Model"), we pass an empty Animation to nyanko.
            let default_anim = Animation::default();
            let animation_to_use = self.current_anim.as_ref().unwrap_or(&default_anim);

            let current_animation_frame = self.current_frame;

            let renderer_lock = self.renderer.clone();
            let rig_clone = rig.clone();
            let anim_clone = animation_to_use.clone();
            let px = self.pan_offset.x;
            let py = self.pan_offset.y;
            let z = self.zoom_level;

            let callback = egui::PaintCallback {
                rect: rect_alloc,
                callback: std::sync::Arc::new(eframe::egui_glow::CallbackFn::new(
                    move |info, painter| {
                        if let Ok(mut renderer_guard) = renderer_lock.lock() {
                            if renderer_guard.is_none() {
                                *renderer_guard = GlowRenderer::new(painter.gl()).ok();
                            }
                            if let Some(renderer) = renderer_guard.as_mut() {
                                let _ = build::frame(
                                    renderer,
                                    painter.gl(),
                                    &rig_clone,
                                    &anim_clone,
                                    current_animation_frame,
                                    info.viewport.width() as f32,
                                    info.viewport.height() as f32,
                                    px, py, z
                                );
                            }
                        }
                    }
                )),
            };
            ui.painter().add(callback);

        } else {
            ui.painter().rect_filled(rect_alloc, 0.0, egui::Color32::from_rgb(20, 20, 20));
        }

        let border_rect = rect_alloc.shrink(2.0);
        let border_color = egui::Color32::from_rgb(31, 106, 165);
        ui.painter().rect_stroke(border_rect, egui::Rounding::same(5.0), egui::Stroke::new(4.0, border_color));

        let controls_hovered = render_controls_overlay(ui, rect_alloc, self, available_anims, base_assets_available, false, secondary_id, primary_id, secondary_assets, interpolation, native_fps, settings);
        self.is_pointer_over_controls = controls_hovered;
    }
}

fn resolve_paths<'a>(
    target_index: usize,
    primary_assets: &'a Option<(PathBuf, PathBuf, PathBuf)>,
    secondary_assets: &'a Option<(PathBuf, PathBuf, PathBuf, PathBuf)>,
    available_anims: &'a [(usize, PathBuf)]
) -> (Option<&'a PathBuf>, Option<&'a PathBuf>, Option<&'a PathBuf>, Option<&'a PathBuf>) {
    if target_index == IDX_SPIRIT {
        if let Some((secondary_png, secondary_cut, secondary_model, secondary_anim)) = secondary_assets {
            return (Some(secondary_png), Some(secondary_cut), Some(secondary_model), Some(secondary_anim));
        }
    } else {
        let animation_path = available_anims.iter().find(|(index, _)| *index == target_index).map(|(_, path)| path);
        if let Some((primary_png, primary_cut, primary_model)) = primary_assets {
            return (Some(primary_png), Some(primary_cut), Some(primary_model), animation_path);
        }
    }
    (None, None, None, None)
}