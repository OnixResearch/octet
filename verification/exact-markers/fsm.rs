#![feature(register_tool)]
#![register_tool(octet, other)]
#[cfg_attr(real_enums, octet::state_machine_enum)]
#[cfg_attr(doc_enums, doc = "octet::state_machine_enum")]
#[cfg_attr(wrong_enums, other::state_machine_enum)]
pub enum State { Ready, Closed }
#[cfg_attr(real_enums, octet::state_machine_enum)]
#[cfg_attr(doc_enums, doc = "state_machine_enum")]
#[cfg_attr(wrong_enums, other::state_machine_enum)]
pub enum Event { Open, Close }
#[cfg_attr(real_core, octet::state_machine_core)]
#[cfg_attr(doc_core, doc = "state_machine_core")]
#[cfg_attr(wrong_core, other::state_machine_core)]
pub fn transition(state: State, event: Event) -> State {
    match (state, event) {
        (State::Ready, Event::Open) => State::Ready,
        (State::Ready, Event::Close) => State::Closed,
        (State::Closed, Event::Open) => State::Ready,
        #[cfg(not(wildcard))]
        (State::Closed, Event::Close) => State::Closed,
        #[cfg(wildcard)]
        _ => State::Closed,
    }
}
