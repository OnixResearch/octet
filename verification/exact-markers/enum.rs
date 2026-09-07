#![feature(register_tool)]
#![register_tool(octet, other)]
#[cfg_attr(real, octet::sealed_enum)]
#[cfg_attr(fsm, octet::state_machine_enum)]
#[cfg_attr(doc, doc = "sealed_enum state_machine_enum octet::sealed_enum")]
#[cfg_attr(wrong, other::sealed_enum)]
#[cfg_attr(payload, octet::unrelated(sealed_enum, "octet::sealed_enum"))]
#[cfg_attr(prefix, octet::sealed_enum_extra)]
#[cfg_attr(nested, octet::nested::sealed_enum)]
pub enum State { Ready, Closed, #[cfg(future)] Future }
pub fn decide(state: State) -> bool {
    match state { State::Ready => true, State::Closed => false }
}
