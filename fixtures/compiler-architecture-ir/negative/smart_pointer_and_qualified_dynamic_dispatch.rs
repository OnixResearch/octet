use std::rc::Rc;
use std::sync::Arc;

pub trait RuntimePort {
    fn invoke(&self);
}

pub struct StaticPort;

impl RuntimePort for StaticPort {
    fn invoke(&self) {}
}

pub fn invoke_boxed(port: Box<dyn RuntimePort>) {
    port.invoke();
}

pub fn invoke_shared(port: Arc<dyn RuntimePort>) {
    port.invoke();
}

pub fn invoke_counted(port: Rc<dyn RuntimePort>) {
    port.invoke();
}

pub fn invoke_qualified(port: &dyn RuntimePort) {
    RuntimePort::invoke(port);
}

pub fn invoke_fully_qualified(port: &dyn RuntimePort) {
    <dyn RuntimePort as RuntimePort>::invoke(port);
}

pub fn invoke_static(port: Box<StaticPort>) {
    port.invoke();
    RuntimePort::invoke(&*port);
}

pub fn invoke_generic<P: RuntimePort>(port: &P) {
    port.invoke();
}
