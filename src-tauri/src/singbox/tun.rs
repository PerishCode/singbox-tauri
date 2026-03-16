#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunState {
    Disabled,
    Passive,
    Selective,
    Full,
}
