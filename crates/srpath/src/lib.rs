use srmath::Vec2;

pub struct Path {
    commands: Vec<PathCommand>,
    closed: bool,
}

pub enum PathCommand {
    MoveTo(Vec2),
    LineTo(Vec2),
    CurveTo(Vec2, Vec2, Vec2),
    Close,
}

impl Path {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            closed: false,
        }
    }

    pub fn move_to(&mut self, p: Vec2) -> &mut Self {
        self.commands.push(PathCommand::MoveTo(p));
        self
    }

    pub fn line_to(&mut self, p: Vec2) -> &mut Self {
        self.commands.push(PathCommand::LineTo(p));
        self
    }

    pub fn curve_to(&mut self, c1: Vec2, c2: Vec2, p: Vec2) -> &mut Self {
        self.commands.push(PathCommand::CurveTo(c1, c2, p));
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.closed = true;
        self.commands.push(PathCommand::Close);
        self
    }

    pub fn commands(&self) -> &[PathCommand] {
        &self.commands
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}
