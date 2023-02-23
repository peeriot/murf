pub trait IntoState {
    type State;

    fn into_state(self) -> Self::State;
}

pub trait FromState<TState, TShared> {
    fn from_state(state: TState, shared: TShared) -> Self;
}
