// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

use crate::services::search_telemetry as telemetry;
use dioxus::core::Task;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct SearchTaskController {
    in_flight: Rc<RefCell<Option<Task>>>,
}

impl SearchTaskController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace_in_flight(&self, next: Task) {
        if let Some(prev) = self.in_flight.borrow_mut().replace(next) {
            prev.cancel();
            telemetry::search_inflight_cancelled();
        }
    }
}
