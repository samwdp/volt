#[derive(Debug, Clone)]
struct StartupTrace {
    origin: Instant,
    last: Instant,
}

impl StartupTrace {
    fn enabled() -> bool {
        std::env::var_os("VOLT_STARTUP_TRACE").is_some()
    }

    fn new() -> Option<Self> {
        let now = Instant::now();
        Self::enabled().then_some(Self {
            origin: now,
            last: now,
        })
    }

    fn mark(&mut self, stage: &str) {
        let now = Instant::now();
        let delta_ms = now.duration_since(self.last).as_secs_f64() * 1000.0;
        let total_ms = now.duration_since(self.origin).as_secs_f64() * 1000.0;
        eprintln!("[startup] {stage}: +{delta_ms:.1}ms total={total_ms:.1}ms");
        self.last = now;
    }
}

fn runtime_pane_rects(
    split_direction: PaneSplitDirection,
    width: u32,
    pane_height: u32,
    pane_count: usize,
    active_pane_index: usize,
    golden_ratio: bool,
    pane_size_weights: Option<&[u32]>,
) -> Vec<PixelRect> {
    if let Some(weights) = pane_size_weights.filter(|weights| !weights.is_empty()) {
        let axis = match split_direction {
            PaneSplitDirection::Vertical => SplitAxis::Columns,
            PaneSplitDirection::Horizontal => SplitAxis::Rows,
        };
        return pane_rects_with_weights(width, pane_height, pane_count, axis, weights);
    }
    match split_direction {
        PaneSplitDirection::Vertical => vertical_pane_rects_for_active(
            width,
            pane_height,
            pane_count,
            active_pane_index,
            golden_ratio,
        ),
        PaneSplitDirection::Horizontal => horizontal_pane_rects_for_active(
            width,
            pane_height,
            pane_count,
            active_pane_index,
            golden_ratio,
        ),
    }
}

fn workspace_pane_rects(
    user_library: &dyn UserLibrary,
    ui: &ShellUiState,
    width: u32,
    pane_height: u32,
    pane_count: usize,
) -> Vec<PixelRect> {
    runtime_pane_rects(
        ui.pane_split_direction(),
        width,
        pane_height,
        pane_count,
        ui.active_pane_index(),
        ui.effective_golden_ratio(user_library),
        ui.pane_size_weights(),
    )
}
