use once_cell::sync::Lazy;

pub struct MockPort {
    state: bool,
}

impl MockPort {
    pub fn get(&self) -> bool {
        self.state
    }

    pub fn set(&mut self, on: bool) {
        self.state = on
    }
}

static mut PORT: Lazy<MockPort> = Lazy::new(|| {
    MockPort {state: false}
});

pub fn unsafe_port() -> &'static mut MockPort {
    unsafe { &mut PORT }
}

pub fn init_port(_args: &crate::Cli) -> std::io::Result<()> {
    println!("Mock implementation!");
    Ok(())
}
