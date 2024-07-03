use std::time::Duration;

use eframe::{egui, NativeOptions};
use egui_task_manager::{executors, Caller, Handle, Task, TaskManager, TasksCollection};

fn main() -> Result<(), eframe::Error> {
    egui_task_manager::setup!();

    eframe::run_native(
        "Task manager example",
        NativeOptions::default(),
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct SimpleCollection;

impl<'c, P> TasksCollection<'c, P> for SimpleCollection {
    type Context = &'c mut u32;

    type Target = u32;

    type Executor = executors::Parallel<P>;

    fn name() -> &'static str {
        "Simple collection"
    }

    fn handle(context: Self::Context) -> egui_task_manager::Handle<'c, Self::Target> {
        Handle::new(|value| *context += value)
    }
}

struct MyApp {
    num: u32,
    task_num: u32,
    task_name: String,
    manager: TaskManager<()>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            manager: TaskManager::new(|_, ()| ()),
            num: 0,
            task_num: 1,
            task_name: "New task".to_owned(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.manager
            .add_collection::<SimpleCollection>(&mut self.num);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.label(format!("The total sum is {}", self.num));
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

            self.manager.ui(ui);
        });
    }
}
