//! L3D 3D Viewer with egui UI - supports multiple viewports

use std::collections::HashMap;
use three_d::*;

/// Result type for loading L3D models
type LoadResult = Result<
    (
        Vec<Model<PhysicalMaterial>>,
        Option<(Vec3, Vec3)>,
        LoadedModelInfo,
    ),
    String,
>;

/// Info about a single loaded L3D file
#[derive(Clone)]
pub struct LoadedModelInfo {
    pub id: usize,
    pub file_name: String,
    pub parts_count: usize,
    #[allow(dead_code)]
    pub assets_count: usize,
}

/// A loaded model with its meshes
pub struct SceneModel {
    pub id: usize,
    pub meshes: Vec<Model<PhysicalMaterial>>,
    pub bounds: Option<(Vec3, Vec3)>,
}

/// A single viewer pane with its own camera and models
pub struct ViewerPane {
    pub id: usize,
    pub camera: Camera,
    pub control: OrbitControl,
    pub models: Vec<SceneModel>,
    pub model_infos: Vec<LoadedModelInfo>,
    pub title: String,
}

impl ViewerPane {
    pub fn new(id: usize, viewport: Viewport) -> Self {
        let camera = Camera::new_perspective(
            viewport,
            vec3(3.0, 3.0, 4.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            1000.0,
        );
        #[allow(clippy::clone_on_copy)]
        let control = OrbitControl::new(camera.target().clone(), 0.5, 50.0);

        Self {
            id,
            camera,
            control,
            models: Vec::new(),
            model_infos: Vec::new(),
            title: format!("Viewer {}", id + 1),
        }
    }

    pub fn total_parts(&self) -> usize {
        self.model_infos.iter().map(|m| m.parts_count).sum()
    }

    pub fn fit_to_bounds(&mut self, viewport: &Viewport) {
        if let Some((min_b, max_b)) = calculate_total_bounds(&self.models) {
            fit_camera(&mut self.camera, &mut self.control, min_b, max_b, viewport);
        }
    }
}

/// View mode - single or multi-viewport
#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    Single,
    Multi,
}

pub struct L3dViewerState {
    pub view_mode: ViewMode,
    pub status: String,
    pub open_requested: bool,
    pub add_to_scene_requested: bool,
    pub add_viewer_requested: bool,
    pub remove_model_requested: Option<(usize, usize)>, // (pane_id, model_id)
    pub remove_pane_requested: Option<usize>,
    pub active_pane: usize,
    pub clear_all_requested: bool,
    next_id: usize,
    next_pane_id: usize,
}

impl Default for L3dViewerState {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Single,
            status: "Drop an L3D file to view".to_string(),
            open_requested: false,
            add_to_scene_requested: false,
            add_viewer_requested: false,
            remove_model_requested: None,
            remove_pane_requested: None,
            active_pane: 0,
            clear_all_requested: false,
            next_id: 0,
            next_pane_id: 0,
        }
    }
}

impl L3dViewerState {
    pub fn next_model_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn next_pane_id(&mut self) -> usize {
        let id = self.next_pane_id;
        self.next_pane_id += 1;
        id
    }
}

/// Calculate grid layout for N panes
fn calculate_grid(count: usize, total_width: u32, total_height: u32) -> Vec<Viewport> {
    if count == 0 {
        return vec![];
    }
    if count == 1 {
        return vec![Viewport::new_at_origo(total_width, total_height)];
    }

    // Calculate grid dimensions (try to be square-ish)
    let cols = (count as f32).sqrt().ceil() as usize;
    let rows = count.div_ceil(cols);

    let cell_width = total_width / cols as u32;
    let cell_height = total_height / rows as u32;

    let mut viewports = Vec::with_capacity(count);
    for i in 0..count {
        let col = i % cols;
        let row = i / cols;
        // Note: OpenGL origin is bottom-left, so we flip y
        let x = (col as u32) * cell_width;
        let y = total_height - ((row as u32) + 1) * cell_height;
        viewports.push(Viewport {
            x: x as i32,
            y: y as i32,
            width: cell_width,
            height: cell_height,
        });
    }
    viewports
}

/// Check if a point is inside a viewport
fn point_in_viewport(x: f32, y: f32, vp: &Viewport, total_height: u32) -> bool {
    // Convert from window coords (top-left origin) to GL coords (bottom-left origin)
    let gl_y = total_height as f32 - y;
    x >= vp.x as f32
        && x < (vp.x + vp.width as i32) as f32
        && gl_y >= vp.y as f32
        && gl_y < (vp.y + vp.height as i32) as f32
}

/// Run the 3D viewer (native) with optional initial content
pub async fn run_viewer(content: Option<Vec<u8>>) {
    let window = Window::new(WindowSettings {
        title: "L3D Viewer".to_string(),
        max_size: Some((1400, 900)),
        ..Default::default()
    })
    .unwrap();

    run_viewer_inner(window, content);
}

/// Run the 3D viewer (WASM) - polls for data from JS
#[cfg(target_arch = "wasm32")]
pub async fn run_viewer_wasm() {
    use wasm_bindgen::JsCast;

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let window = Window::new(WindowSettings {
        title: "L3D Viewer".to_string(),
        canvas: Some(canvas),
        ..Default::default()
    })
    .unwrap();

    run_viewer_wasm_inner(window);
}

#[cfg(target_arch = "wasm32")]
fn run_viewer_wasm_inner(window: Window) {
    use std::cell::RefCell;
    use std::rc::Rc;

    let context = window.gl();
    let mut gui = three_d::GUI::new(&context);
    let mut state = L3dViewerState::default();

    let light0 = DirectionalLight::new(&context, 2.0, Srgba::WHITE, vec3(-1.0, -1.0, -1.0));
    let light1 = DirectionalLight::new(&context, 0.5, Srgba::WHITE, vec3(1.0, 0.5, 0.5));
    let ambient = AmbientLight::new(&context, 0.4, Srgba::WHITE);

    // Start with one empty pane
    let mut panes: Vec<ViewerPane> = vec![ViewerPane::new(state.next_pane_id(), window.viewport())];

    // Track load mode: 0=replace in active, 1=add to active, 2=new viewer per file
    let load_mode = Rc::new(RefCell::new(0u8));

    window.render_loop(move |mut frame_input| {
        let total_vp = frame_input.viewport;

        // Check for pending data from JavaScript
        if let Some(data) = crate::take_pending_data() {
            state.status = "Loading...".to_string();
            let mode = *load_mode.borrow();
            match load_l3d_models(&context, &data, None) {
                Ok((meshes, bounds, mut info)) => {
                    info.id = state.next_model_id();

                    match mode {
                        0 => {
                            // Replace mode - clear active pane
                            let pane_count = panes.len();
                            if let Some(pane) = panes.get_mut(state.active_pane) {
                                pane.models.clear();
                                pane.model_infos.clear();
                                pane.model_infos.push(info.clone());
                                pane.models.push(SceneModel {
                                    id: info.id,
                                    meshes,
                                    bounds,
                                });
                                pane.title = info.file_name.clone();
                                let vps =
                                    calculate_grid(pane_count, total_vp.width, total_vp.height);
                                if let Some(vp) = vps.get(state.active_pane) {
                                    pane.fit_to_bounds(vp);
                                }
                            }
                            state.status = format!("Loaded: {}", info.file_name);
                        }
                        1 => {
                            // Add to scene mode
                            let pane_count = panes.len();
                            if let Some(pane) = panes.get_mut(state.active_pane) {
                                pane.model_infos.push(info.clone());
                                pane.models.push(SceneModel {
                                    id: info.id,
                                    meshes,
                                    bounds,
                                });
                                let vps =
                                    calculate_grid(pane_count, total_vp.width, total_vp.height);
                                if let Some(vp) = vps.get(state.active_pane) {
                                    pane.fit_to_bounds(vp);
                                }
                            }
                            state.status = format!("Added: {}", info.file_name);
                        }
                        2 => {
                            // New viewer mode
                            state.view_mode = ViewMode::Multi;
                            let pane_id = state.next_pane_id();
                            let mut new_pane = ViewerPane::new(pane_id, total_vp);
                            new_pane.title = info.file_name.clone();
                            new_pane.model_infos.push(info.clone());
                            new_pane.models.push(SceneModel {
                                id: info.id,
                                meshes,
                                bounds,
                            });
                            panes.push(new_pane);
                            state.active_pane = panes.len() - 1;
                            // Fit camera after grid recalculation
                            let vps = calculate_grid(panes.len(), total_vp.width, total_vp.height);
                            if let Some(vp) = vps.get(state.active_pane) {
                                if let Some(pane) = panes.get_mut(state.active_pane) {
                                    pane.fit_to_bounds(vp);
                                }
                            }
                            state.status = format!("New viewer: {}", info.file_name);
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    state.status = format!("Error: {}", e);
                    log::error!("Load error: {}", e);
                }
            }
            *load_mode.borrow_mut() = 0;
        }

        // Calculate viewport grid
        let viewports = calculate_grid(panes.len(), total_vp.width, total_vp.height);

        // GUI
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |egui_ctx| {
                render_ui(egui_ctx, &mut state, &panes);
            },
        );

        // Handle file open request (replace mode)
        if state.open_requested {
            state.open_requested = false;
            *load_mode.borrow_mut() = 0;
            wasm_bindgen_futures::spawn_local(async {
                if let Some(file) = rfd::AsyncFileDialog::new()
                    .add_filter("L3D files", &["l3d"])
                    .pick_file()
                    .await
                {
                    let data = file.read().await;
                    crate::PENDING_DATA.with(|d| {
                        *d.borrow_mut() = Some(data);
                    });
                }
            });
        }

        // Handle add to scene request
        if state.add_to_scene_requested {
            state.add_to_scene_requested = false;
            *load_mode.borrow_mut() = 1;
            wasm_bindgen_futures::spawn_local(async {
                if let Some(file) = rfd::AsyncFileDialog::new()
                    .add_filter("L3D files", &["l3d"])
                    .pick_file()
                    .await
                {
                    let data = file.read().await;
                    crate::PENDING_DATA.with(|d| {
                        *d.borrow_mut() = Some(data);
                    });
                }
            });
        }

        // Handle add viewer request
        if state.add_viewer_requested {
            state.add_viewer_requested = false;
            *load_mode.borrow_mut() = 2;
            wasm_bindgen_futures::spawn_local(async {
                if let Some(file) = rfd::AsyncFileDialog::new()
                    .add_filter("L3D files", &["l3d"])
                    .pick_file()
                    .await
                {
                    let data = file.read().await;
                    crate::PENDING_DATA.with(|d| {
                        *d.borrow_mut() = Some(data);
                    });
                }
            });
        }

        // Handle remove model request
        if let Some((pane_id, model_id)) = state.remove_model_requested.take() {
            if let Some(pane) = panes.iter_mut().find(|p| p.id == pane_id) {
                pane.models.retain(|m| m.id != model_id);
                pane.model_infos.retain(|m| m.id != model_id);
                if let Some(vp) = viewports.iter().find(|_| true) {
                    pane.fit_to_bounds(vp);
                }
            }
            state.status = "Model removed".to_string();
        }

        // Handle remove pane request
        if let Some(pane_id) = state.remove_pane_requested.take() {
            panes.retain(|p| p.id != pane_id);
            if panes.is_empty() {
                panes.push(ViewerPane::new(state.next_pane_id(), total_vp));
                state.view_mode = ViewMode::Single;
            }
            if state.active_pane >= panes.len() {
                state.active_pane = panes.len().saturating_sub(1);
            }
            state.status = "Viewer closed".to_string();
        }

        // Handle clear all
        if state.clear_all_requested {
            state.clear_all_requested = false;
            panes.clear();
            panes.push(ViewerPane::new(state.next_pane_id(), total_vp));
            state.view_mode = ViewMode::Single;
            state.active_pane = 0;
            state.status = "All cleared".to_string();
        }

        // Determine which pane is hovered for input
        let mut hovered_pane = state.active_pane;
        for event in &frame_input.events {
            if let Event::MouseMotion { position, .. } = event {
                for (i, vp) in viewports.iter().enumerate() {
                    if point_in_viewport(position.x, position.y, vp, total_vp.height) {
                        hovered_pane = i;
                        break;
                    }
                }
            }
            if let Event::MousePress { .. } = event {
                state.active_pane = hovered_pane;
            }
        }

        // Update cameras and handle input for hovered pane
        for (i, pane) in panes.iter_mut().enumerate() {
            if let Some(vp) = viewports.get(i) {
                pane.camera.set_viewport(*vp);
                if i == hovered_pane {
                    pane.control
                        .handle_events(&mut pane.camera, &mut frame_input.events);
                }
            }
        }

        // Render
        let screen = frame_input.screen();
        screen.clear(ClearState::color_and_depth(0.15, 0.15, 0.2, 1.0, 1.0));

        for (i, pane) in panes.iter().enumerate() {
            if let Some(vp) = viewports.get(i) {
                screen
                    .clear_partially(ScissorBox::from(*vp), ClearState::depth(1.0))
                    .render_partially(
                        ScissorBox::from(*vp),
                        &pane.camera,
                        pane.models.iter().flat_map(|sm| sm.meshes.iter().flatten()),
                        &[&light0, &light1, &ambient],
                    );
            }
        }

        screen.write(|| gui.render()).unwrap();

        FrameOutput::default()
    });
}

fn run_viewer_inner(window: Window, content: Option<Vec<u8>>) {
    let context = window.gl();
    let mut gui = three_d::GUI::new(&context);
    let mut state = L3dViewerState::default();

    let light0 = DirectionalLight::new(&context, 2.0, Srgba::WHITE, vec3(-1.0, -1.0, -1.0));
    let light1 = DirectionalLight::new(&context, 0.5, Srgba::WHITE, vec3(1.0, 0.5, 0.5));
    let ambient = AmbientLight::new(&context, 0.4, Srgba::WHITE);

    // Start with one pane
    let mut panes: Vec<ViewerPane> = vec![ViewerPane::new(state.next_pane_id(), window.viewport())];

    // Load initial content
    if let Some(data) = content {
        match load_l3d_models(&context, &data, None) {
            Ok((meshes, bounds, mut info)) => {
                info.id = state.next_model_id();
                if let Some(pane) = panes.get_mut(0) {
                    pane.model_infos.push(info.clone());
                    pane.models.push(SceneModel {
                        id: info.id,
                        meshes,
                        bounds,
                    });
                    pane.title = info.file_name.clone();
                    pane.fit_to_bounds(&window.viewport());
                }
                state.status = format!("Loaded: {}", info.file_name);
            }
            Err(e) => {
                state.status = format!("Error: {}", e);
            }
        }
    }

    window.render_loop(move |mut frame_input| {
        let total_vp = frame_input.viewport;

        // Calculate viewport grid
        let viewports = calculate_grid(panes.len(), total_vp.width, total_vp.height);

        // GUI
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |egui_ctx| {
                render_ui(egui_ctx, &mut state, &panes);
            },
        );

        // Handle file open request (replace mode)
        #[cfg(not(target_arch = "wasm32"))]
        if state.open_requested {
            state.open_requested = false;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("L3D files", &["l3d"])
                .pick_file()
            {
                if let Ok(data) = std::fs::read(&path) {
                    state.status = "Loading...".to_string();
                    let file_name = path.file_name().map(|n| n.to_string_lossy().to_string());
                    match load_l3d_models(&context, &data, file_name) {
                        Ok((meshes, bounds, mut info)) => {
                            info.id = state.next_model_id();
                            if let Some(pane) = panes.get_mut(state.active_pane) {
                                pane.models.clear();
                                pane.model_infos.clear();
                                pane.model_infos.push(info.clone());
                                pane.models.push(SceneModel {
                                    id: info.id,
                                    meshes,
                                    bounds,
                                });
                                pane.title = info.file_name.clone();
                                if let Some(vp) = viewports.get(state.active_pane) {
                                    pane.fit_to_bounds(vp);
                                }
                            }
                            state.status = format!("Loaded: {}", info.file_name);
                        }
                        Err(e) => {
                            state.status = format!("Error: {}", e);
                        }
                    }
                }
            }
        }

        // Handle add to scene request
        #[cfg(not(target_arch = "wasm32"))]
        if state.add_to_scene_requested {
            state.add_to_scene_requested = false;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("L3D files", &["l3d"])
                .pick_file()
            {
                if let Ok(data) = std::fs::read(&path) {
                    state.status = "Loading...".to_string();
                    let file_name = path.file_name().map(|n| n.to_string_lossy().to_string());
                    match load_l3d_models(&context, &data, file_name) {
                        Ok((meshes, bounds, mut info)) => {
                            info.id = state.next_model_id();
                            if let Some(pane) = panes.get_mut(state.active_pane) {
                                pane.model_infos.push(info.clone());
                                pane.models.push(SceneModel {
                                    id: info.id,
                                    meshes,
                                    bounds,
                                });
                                if let Some(vp) = viewports.get(state.active_pane) {
                                    pane.fit_to_bounds(vp);
                                }
                            }
                            state.status = format!("Added: {}", info.file_name);
                        }
                        Err(e) => {
                            state.status = format!("Error: {}", e);
                        }
                    }
                }
            }
        }

        // Handle add viewer request
        #[cfg(not(target_arch = "wasm32"))]
        if state.add_viewer_requested {
            state.add_viewer_requested = false;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("L3D files", &["l3d"])
                .pick_file()
            {
                if let Ok(data) = std::fs::read(&path) {
                    state.status = "Loading...".to_string();
                    let file_name = path.file_name().map(|n| n.to_string_lossy().to_string());
                    match load_l3d_models(&context, &data, file_name) {
                        Ok((meshes, bounds, mut info)) => {
                            info.id = state.next_model_id();
                            state.view_mode = ViewMode::Multi;
                            let pane_id = state.next_pane_id();
                            let mut new_pane = ViewerPane::new(pane_id, total_vp);
                            new_pane.title = info.file_name.clone();
                            new_pane.model_infos.push(info.clone());
                            new_pane.models.push(SceneModel {
                                id: info.id,
                                meshes,
                                bounds,
                            });
                            panes.push(new_pane);
                            state.active_pane = panes.len() - 1;
                            // Recalculate viewports and fit
                            let new_vps =
                                calculate_grid(panes.len(), total_vp.width, total_vp.height);
                            if let Some(vp) = new_vps.get(state.active_pane) {
                                if let Some(pane) = panes.get_mut(state.active_pane) {
                                    pane.fit_to_bounds(vp);
                                }
                            }
                            state.status = format!("New viewer: {}", info.file_name);
                        }
                        Err(e) => {
                            state.status = format!("Error: {}", e);
                        }
                    }
                }
            }
        }

        // Handle remove model request
        if let Some((pane_id, model_id)) = state.remove_model_requested.take() {
            if let Some((i, pane)) = panes.iter_mut().enumerate().find(|(_, p)| p.id == pane_id) {
                pane.models.retain(|m| m.id != model_id);
                pane.model_infos.retain(|m| m.id != model_id);
                if let Some(vp) = viewports.get(i) {
                    pane.fit_to_bounds(vp);
                }
            }
            state.status = "Model removed".to_string();
        }

        // Handle remove pane request
        if let Some(pane_id) = state.remove_pane_requested.take() {
            panes.retain(|p| p.id != pane_id);
            if panes.is_empty() {
                panes.push(ViewerPane::new(state.next_pane_id(), total_vp));
                state.view_mode = ViewMode::Single;
            }
            if state.active_pane >= panes.len() {
                state.active_pane = panes.len().saturating_sub(1);
            }
            state.status = "Viewer closed".to_string();
        }

        // Handle clear all
        if state.clear_all_requested {
            state.clear_all_requested = false;
            panes.clear();
            panes.push(ViewerPane::new(state.next_pane_id(), total_vp));
            state.view_mode = ViewMode::Single;
            state.active_pane = 0;
            state.status = "All cleared".to_string();
        }

        // Determine which pane is hovered for input
        let mut hovered_pane = state.active_pane;
        for event in &frame_input.events {
            if let Event::MouseMotion { position, .. } = event {
                for (i, vp) in viewports.iter().enumerate() {
                    if point_in_viewport(position.x, position.y, vp, total_vp.height) {
                        hovered_pane = i;
                        break;
                    }
                }
            }
            if let Event::MousePress { .. } = event {
                state.active_pane = hovered_pane;
            }
        }

        // Update cameras and handle input for hovered pane
        for (i, pane) in panes.iter_mut().enumerate() {
            if let Some(vp) = viewports.get(i) {
                pane.camera.set_viewport(*vp);
                if i == hovered_pane {
                    pane.control
                        .handle_events(&mut pane.camera, &mut frame_input.events);
                }
            }
        }

        // Render
        let screen = frame_input.screen();
        screen.clear(ClearState::color_and_depth(0.15, 0.15, 0.2, 1.0, 1.0));

        for (i, pane) in panes.iter().enumerate() {
            if let Some(vp) = viewports.get(i) {
                screen
                    .clear_partially(ScissorBox::from(*vp), ClearState::depth(1.0))
                    .render_partially(
                        ScissorBox::from(*vp),
                        &pane.camera,
                        pane.models.iter().flat_map(|sm| sm.meshes.iter().flatten()),
                        &[&light0, &light1, &ambient],
                    );
            }
        }

        screen.write(|| gui.render()).unwrap();

        FrameOutput::default()
    });
}

fn render_ui(ctx: &three_d::egui::Context, state: &mut L3dViewerState, panes: &[ViewerPane]) {
    use three_d::egui::*;

    // Top menu bar
    TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open...").clicked() {
                    state.open_requested = true;
                    ui.close_menu();
                }
                if ui.button("Add to Scene...").clicked() {
                    state.add_to_scene_requested = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Open in New Viewer...").clicked() {
                    state.add_viewer_requested = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Clear All").clicked() {
                    state.clear_all_requested = true;
                    ui.close_menu();
                }
                ui.separator();
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });
            ui.menu_button("View", |ui| {
                if ui
                    .radio_value(&mut state.view_mode, ViewMode::Single, "Single Viewer")
                    .clicked()
                {
                    ui.close_menu();
                }
                if ui
                    .radio_value(&mut state.view_mode, ViewMode::Multi, "Multi Viewer")
                    .clicked()
                {
                    ui.close_menu();
                }
            });
            ui.menu_button("Help", |ui| {
                ui.label("Drag: Rotate camera");
                ui.label("Scroll: Zoom");
                ui.label("Click viewport to select");
            });
        });
    });

    // Status bar
    TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(&state.status);
            ui.separator();
            let total_models: usize = panes.iter().map(|p| p.model_infos.len()).sum();
            let total_parts: usize = panes.iter().map(|p| p.total_parts()).sum();
            ui.label(format!(
                "{} viewer(s), {} model(s), {} parts",
                panes.len(),
                total_models,
                total_parts
            ));
        });
    });

    // Side panel with info
    SidePanel::left("info_panel")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("L3D Viewer");
            ui.separator();

            if panes.is_empty() || panes.iter().all(|p| p.model_infos.is_empty()) {
                ui.label("No models loaded");
                ui.separator();
                ui.label("• File > Open: Load in current viewer");
                ui.label("• File > Add to Scene: Add model");
                ui.label("• File > Open in New Viewer: Create new viewport");
            } else {
                ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    for (pane_idx, pane) in panes.iter().enumerate() {
                        let is_active = pane_idx == state.active_pane;
                        let header = if is_active {
                            format!("► {} (active)", pane.title)
                        } else {
                            pane.title.clone()
                        };

                        CollapsingHeader::new(header)
                            .default_open(is_active)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "{} model(s), {} parts",
                                        pane.model_infos.len(),
                                        pane.total_parts()
                                    ));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if panes.len() > 1 && ui.small_button("Close").clicked() {
                                            state.remove_pane_requested = Some(pane.id);
                                        }
                                    });
                                });

                                for model in &pane.model_infos {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("  • {}", model.file_name));
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                if ui.small_button("X").clicked() {
                                                    state.remove_model_requested =
                                                        Some((pane.id, model.id));
                                                }
                                            },
                                        );
                                    });
                                }
                            });
                    }
                });
            }

            ui.separator();
            ui.heading("Controls");
            ui.label("• Drag: Rotate");
            ui.label("• Scroll: Zoom");
            ui.label("• Shift+Drag: Pan");
            ui.label("• Click: Select viewer");
        });
}

fn load_l3d_models(context: &Context, content: &[u8], file_name: Option<String>) -> LoadResult {
    let l3d = l3d_rs::from_buffer(content);

    if l3d.model.parts.is_empty() {
        return Err("No parts found".to_string());
    }

    let parts_count = l3d.model.parts.len();
    let assets_count = l3d.file.assets.len();

    let display_name = file_name.unwrap_or_else(|| {
        l3d.model
            .parts
            .first()
            .map(|p| p.path.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    });

    log::info!("L3D: {} parts, {} assets", parts_count, assets_count);

    let mut raw_assets = three_d_asset::io::RawAssets::new();
    for asset in &l3d.file.assets {
        log::info!("Asset: {} ({} bytes)", asset.name, asset.content.len());
        raw_assets.insert(&asset.name, asset.content.clone());
    }
    add_stub_mtls(&l3d, &mut raw_assets);

    let rotation = Mat4::from_angle_x(Deg(-90.0));
    let mut cpu_models: HashMap<String, CpuModel> = HashMap::new();
    let mut models: Vec<Model<PhysicalMaterial>> = Vec::new();
    let mut min_b = vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max_b = vec3(f32::MIN, f32::MIN, f32::MIN);

    for part in &l3d.model.parts {
        let cpu_mdl = match cpu_models.get(&part.path) {
            Some(m) => m,
            None => match raw_assets.deserialize::<CpuModel>(&part.path) {
                Ok(m) => {
                    cpu_models.insert(part.path.clone(), m);
                    cpu_models.get(&part.path).unwrap()
                }
                Err(e) => {
                    log::warn!("Skip '{}': {:?}", part.path, e);
                    continue;
                }
            },
        };

        if let Ok(mut mdl) = Model::<PhysicalMaterial>::new(context, cpu_mdl) {
            let part_mat = Mat4::from_cols(
                vec4(part.mat[0], part.mat[1], part.mat[2], part.mat[3]),
                vec4(part.mat[4], part.mat[5], part.mat[6], part.mat[7]),
                vec4(part.mat[8], part.mat[9], part.mat[10], part.mat[11]),
                vec4(part.mat[12], part.mat[13], part.mat[14], part.mat[15]),
            );
            let mat = rotation * part_mat;

            mdl.iter_mut().for_each(|m| {
                m.set_transformation(mat);
                let aabb = m.aabb();
                min_b.x = min_b.x.min(aabb.min().x);
                min_b.y = min_b.y.min(aabb.min().y);
                min_b.z = min_b.z.min(aabb.min().z);
                max_b.x = max_b.x.max(aabb.max().x);
                max_b.y = max_b.y.max(aabb.max().y);
                max_b.z = max_b.z.max(aabb.max().z);
            });
            models.push(mdl);
        }
    }

    if models.is_empty() {
        return Err("No models loaded".to_string());
    }

    let bounds = if min_b.x < max_b.x {
        Some((min_b, max_b))
    } else {
        None
    };
    let info = LoadedModelInfo {
        id: 0,
        file_name: display_name,
        parts_count,
        assets_count,
    };
    Ok((models, bounds, info))
}

fn calculate_total_bounds(scene_models: &[SceneModel]) -> Option<(Vec3, Vec3)> {
    let mut min_b = vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max_b = vec3(f32::MIN, f32::MIN, f32::MIN);
    let mut has_bounds = false;

    for sm in scene_models {
        if let Some((sm_min, sm_max)) = sm.bounds {
            has_bounds = true;
            min_b.x = min_b.x.min(sm_min.x);
            min_b.y = min_b.y.min(sm_min.y);
            min_b.z = min_b.z.min(sm_min.z);
            max_b.x = max_b.x.max(sm_max.x);
            max_b.y = max_b.y.max(sm_max.y);
            max_b.z = max_b.z.max(sm_max.z);
        }
    }

    if has_bounds {
        Some((min_b, max_b))
    } else {
        None
    }
}

fn fit_camera(
    camera: &mut Camera,
    control: &mut OrbitControl,
    min_b: Vec3,
    max_b: Vec3,
    vp: &Viewport,
) {
    let center = (min_b + max_b) / 2.0;
    let size = max_b - min_b;
    let max_dim = size.x.max(size.y).max(size.z);
    let dist = max_dim / (2.0 * 22.5_f32.to_radians().tan());
    let pos = center + vec3(dist * 0.7, dist * 0.7, dist);

    *camera = Camera::new_perspective(
        *vp,
        pos,
        center,
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.01,
        dist * 10.0,
    );
    *control = OrbitControl::new(center, dist * 0.1, dist * 5.0);
}

fn add_stub_mtls(l3d: &l3d_rs::L3d, raw: &mut three_d_asset::io::RawAssets) {
    const STUB_MTL: &[u8] = b"# Stub material
newmtl default
Ns 100.0
Ka 0.2 0.2 0.2
Kd 0.8 0.8 0.8
Ks 0.1 0.1 0.1
d 1.0
illum 2
";

    for asset in &l3d.file.assets {
        if asset.name.to_lowercase().ends_with(".obj") {
            if let Ok(content) = std::str::from_utf8(&asset.content) {
                let base = asset
                    .name
                    .rfind('/')
                    .map(|p| &asset.name[..=p])
                    .unwrap_or("");
                for line in content.lines() {
                    if let Some(mtl) = line.trim().strip_prefix("mtllib ") {
                        let path = format!("{}{}", base, mtl.trim());
                        if !l3d.file.assets.iter().any(|a| a.name == path) {
                            log::info!("Adding stub MTL: {}", path);
                            raw.insert(&path, STUB_MTL.to_vec());
                        }
                    }
                }
            }
        }
    }
}
