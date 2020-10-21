/* LICENSE BEGIN
    This file is part of the SixtyFPS Project -- https://sixtyfps.io
    Copyright (c) 2020 Olivier Goffart <olivier.goffart@sixtyfps.io>
    Copyright (c) 2020 Simon Hausmann <simon.hausmann@sixtyfps.io>

    SPDX-License-Identifier: GPL-3.0-only
    This file is also available under commercial licensing terms.
    Please contact info@sixtyfps.io for more information.
LICENSE END */

use core::pin;
use pin::Pin;
use pin_weak::rc::PinWeak;
use sixtyfps::Model;
use std::{cell::RefCell, rc::Rc};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

sixtyfps::include_modules!();

struct TodoModel {
    items: RefCell<Vec<TodoItem>>,
    main_window: PinWeak<MainWindow>,
    notify: sixtyfps::ModelNotify,
}

impl sixtyfps::Model for TodoModel {
    type Data = TodoItem;

    fn row_count(&self) -> usize {
        self.items.borrow().len()
    }

    fn row_data(&self, row: usize) -> Self::Data {
        self.items.borrow()[row].clone()
    }

    fn set_row_data(&self, row: usize, data: Self::Data) {
        let mut items = self.items.borrow_mut();
        let existing_item = &mut items[row];
        match (existing_item.checked, data.checked) {
            (true, false) => self.adjust_todo_count(1),
            (false, true) => self.adjust_todo_count(-1),
            _ => {}
        };

        *existing_item = data;
        self.notify.row_changed(row);
    }

    fn attach_peer(&self, peer: sixtyfps::ModelPeer) {
        self.notify.attach(peer);
    }
}

impl TodoModel {
    fn new(data: Vec<TodoItem>, main_window: &Pin<Rc<MainWindow>>) -> Self {
        let todo_left_count =
            data.iter().fold(0, |count, item| if item.checked { count } else { count + 1 });
        main_window.as_ref().set_todo_left(todo_left_count);
        Self {
            items: RefCell::new(data),
            main_window: PinWeak::downgrade(main_window.clone()),
            notify: Default::default(),
        }
    }

    pub fn push(&self, value: TodoItem) {
        if !value.checked {
            self.adjust_todo_count(1);
        }
        self.items.borrow_mut().push(value);
        self.notify.row_added(self.items.borrow().len(), 1);
    }

    pub fn remove(&self, index: usize) {
        let removed_item = self.items.borrow_mut().remove(index);
        if !removed_item.checked {
            self.adjust_todo_count(-1)
        }
        self.notify.row_removed(index, 1);
    }

    fn adjust_todo_count(&self, adjustment: i32) {
        let main_window = self.main_window.upgrade().unwrap();
        main_window.as_ref().set_todo_left(main_window.as_ref().get_todo_left() + adjustment);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    let main_window = MainWindow::new();

    let todo_model = Rc::new(TodoModel::new(
        vec![
            TodoItem { checked: true, title: "Implement the .60 file".into() },
            TodoItem { checked: true, title: "Do the Rust part".into() },
            TodoItem { checked: false, title: "Make the C++ code".into() },
            TodoItem { checked: false, title: "Write some JavaScript code".into() },
            TodoItem { checked: false, title: "Test the application".into() },
            TodoItem { checked: false, title: "Ship to customer".into() },
            TodoItem { checked: false, title: "???".into() },
            TodoItem { checked: false, title: "Profit".into() },
        ],
        &main_window,
    ));

    main_window.as_ref().on_todo_added({
        let todo_model = todo_model.clone();
        move |text| todo_model.push(TodoItem { checked: false, title: text })
    });
    main_window.as_ref().on_remove_done({
        let todo_model = todo_model.clone();
        move || {
            let mut offset = 0;
            for i in 0..todo_model.row_count() {
                if todo_model.row_data(i - offset).checked {
                    todo_model.remove(i - offset);
                    offset += 1;
                }
            }
        }
    });

    main_window.as_ref().set_todo_model(Some(todo_model));

    main_window.run();
}
