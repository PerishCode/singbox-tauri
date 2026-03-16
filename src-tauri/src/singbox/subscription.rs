#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionState {
    Missing,
    Ready,
    Failed,
}
