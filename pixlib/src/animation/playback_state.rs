#[derive(Clone, Debug, Default, PartialEq, Eq, Copy)]
pub enum PlaybackState {
    #[default]
    Forward,
    Backward,
    ForwardPaused,
    BackwardPaused,
    Stopped,
}
