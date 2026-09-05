#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameDirection {
    Sent,
    Received,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub direction: FrameDirection,
    pub line: String,
}
