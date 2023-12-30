#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum PlaybackState {
    Forward,
    Backward,
    ForwardPaused,
    BackwardPaused,
    Stopped,
}
