pub mod adapters {
    pub struct Http {
        pub base: String,
    }

    pub struct SystemClock;

    pub enum Transport {
        Loopback,
    }
}

pub fn compose(base: String) -> (adapters::Http, adapters::SystemClock, adapters::Transport) {
    (adapters::Http { base }, adapters::SystemClock, adapters::Transport::Loopback)
}
