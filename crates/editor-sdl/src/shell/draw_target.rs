enum DrawTarget<'a> {
    Scene(&'a mut Vec<DrawCommand>),
}

impl DrawTarget<'_> {
    fn clear(&mut self, color: Color) {
        match self {
            Self::Scene(scene) => scene.push(DrawCommand::Clear {
                color: to_render_color(color),
            }),
        }
    }

    fn scene(&mut self) -> &mut Vec<DrawCommand> {
        match self {
            Self::Scene(scene) => scene,
        }
    }
}
