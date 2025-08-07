#![no_std]
#![allow(static_mut_refs)]

use sails_rs::prelude::*;

struct PingPongService(());

impl PingPongService {
    pub fn new() -> Self {
        Self(())
    }
}

#[sails_rs::service]
impl PingPongService {
    fn init() -> Self {
        Self(())
    }

    pub fn ping(&mut self) -> String {
        String::from("Pong!")
    }
}

pub struct PingPongProgram(());

#[allow(clippy::new_without_default)]
#[sails_rs::program]
impl PingPongProgram {
    pub fn new() -> Self {
        PingPongService::init();
        Self(())
    }

    pub fn ping_pong(&self) -> PingPongService {
        PingPongService::new()
    }
}
