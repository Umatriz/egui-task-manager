//! An example of how you can use this crate to increment a counter.
//!
//! Obviously in a real project you don't want to use this crate just for a counter.

use std::time::Duration;

use eframe::{egui, NativeOptions};
use egui_task_manager::{executors, Caller, Handler, Progress, Task, TaskManager, TasksCollection};

fn main() -> Result<(), eframe::Error> {
    egui_task_manager::setup!();

    eframe::run_native(
        "Task manager example",
        NativeOptions::default(),
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

struct SimpleCollection;

impl<'c> TasksCollection<'c> for SimpleCollection {
    type Context = &'c mut u32;

    type Target = u32;

    type Executor = executors::Parallel;

    fn name() -> &'static str {
        "Simple collection"
    }

    fn handle(context: Self::Context) -> egui_task_manager::Handler<'c, Self::Target> {
        Handler::new(|value| *context += value)
    }
}

struct LabelCollection;

impl<'c> TasksCollection<'c> for LabelCollection {
    type Context = &'c mut String;

    type Target = String;

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Update label collection"
    }

    fn handle(context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|value| *context = value)
    }
}

struct UnitProgress;

impl Progress for UnitProgress {
    fn apply(&self, current: &mut u32) {
        *current += 1;
    }
}

struct MyApp {
    manager: TaskManager,

    num: u32,
    task_num: u32,
    task_name: String,

    current_label: String,
    label_to_set: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            manager: TaskManager::new(),
            num: 0,
            task_num: 1,
            task_name: "New task".to_owned(),
            current_label: "default label".to_owned(),
            label_to_set: "new label".to_owned(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.manager
            .add_collection::<SimpleCollection>(&mut self.num)
            .add_collection::<LabelCollection>(&mut self.current_label);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.label(format!("The total sum is {}", self.num));
                ui.label(format!("Current label is: {}", self.current_label));
            });

            ui.group(|ui| {
                ui.add(egui::DragValue::new(&mut self.task_num));
                ui.text_edit_singleline(&mut self.task_name);

                if ui.button("Add a new task").on_hover_text("It will spawn a task that will wait for 2 seconds and then return the number above").clicked() {
                    let num = self.task_num;
                    self.manager.push_task::<SimpleCollection>(Task::new(
                        self.task_name.clone(),
                        Caller::standard(async move {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            num
                        }),
                    ))
                };
            });

            ui.group(|ui| {
                ui.label("Task's name and new label:");
                ui.text_edit_singleline(&mut self.label_to_set);
                if ui.button("Add a new label!").clicked(){
                    let label = self.label_to_set.clone();
                    let caller = Caller::progressing(|progress| async move {
                        let _ = progress.set_total(10);
                        for _ in 0..10 {
                            let _ = progress.update(UnitProgress);
                            tokio::time::sleep(Duration::from_secs_f32(0.5)).await;
                        }
                        label
                    });
                    self.manager.push_task::<LabelCollection>(Task::new(&self.label_to_set, caller));
                }
            });

            self.manager.ui(ui);
        });
    }
}
