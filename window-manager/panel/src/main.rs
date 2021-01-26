extern crate chrono;
extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;

use std::{env::args, error::Error};

use calendar::{day_of_week, days_in_month};
use chrono::{Datelike, Local};
use gdk::WindowTypeHint;
use gio::prelude::*;
use gtk::{
    prelude::*, Application, ApplicationWindow, AspectFrame, Builder, Button, Label, StyleContext,
};

mod calendar;

const RESOURCE_PREFIX: &str = "/co/dothq/os/panel";

const PADDING: i32 = 8;
const HEIGHT: i32 = 16;

fn resource(name: &str) -> String {
    format!("{}/{}", RESOURCE_PREFIX, name)
}

struct Panel {
    window: ApplicationWindow,
    start_menu: ApplicationWindow,
    calendar: ApplicationWindow,
    builder: Builder,
}

impl Panel {
    fn new(app: &Application) -> Self {
        // Initialize the window
        let builder = Builder::from_resource(&resource("panel.glade"));
        let window: ApplicationWindow = builder.get_object("panel").unwrap();

        window.set_application(Some(app));
        window.set_skip_taskbar_hint(true);
        window.set_decorated(false);
        window.set_type_hint(WindowTypeHint::Dock);

        // Start menu window
        let start_menu: ApplicationWindow = builder.get_object("start_menu").unwrap();
        start_menu.set_application(Some(app));
        start_menu.set_skip_taskbar_hint(true);
        start_menu.set_type_hint(WindowTypeHint::Dock);

        let calendar: ApplicationWindow = builder.get_object("calendar").unwrap();
        start_menu.set_application(Some(app));
        calendar.set_type_hint(WindowTypeHint::Dock);

        Panel {
            window,
            builder,
            calendar,
            start_menu,
        }
    }

    fn pin(&self) {
        self.build_calendar();

        let screen = self.window.get_screen().unwrap();

        let current_monitor = screen.get_display().get_primary_monitor().unwrap();
        let current_monitor_geometry = current_monitor.get_geometry();

        let x = current_monitor_geometry.x;
        let width = current_monitor_geometry.width;
        let height = current_monitor_geometry.height;

        self.window.move_(x + PADDING, height - HEIGHT - 32);
        self.window.resize(width - PADDING * 2, HEIGHT);

        let start_menu_size = self.start_menu.get_size();

        self.start_menu.move_(
            x + PADDING,
            height - HEIGHT - start_menu_size.1 * 2 - PADDING,
        );

        self.window.show_all();
        self.start_menu.show_all();
        self.calendar.show_all();

        let calendar_menu_size = self.calendar.get_size();

        self.calendar.move_(
            width - PADDING - calendar_menu_size.0,
            height - HEIGHT - calendar_menu_size.1 - PADDING * 5,
        );

        self.start_menu.set_visible(false);
        self.calendar.set_visible(false);
    }

    fn build_calendar(&self) {
        let date = Local::now().date().naive_local();

        let month = 1;
        let year = 2021;
        let day = date.day() as usize;

        let headers = vec!["S", "M", "T", "W", "T", "F", "S"]
            .iter()
            .map(|e| (format!("{} ", e), false))
            .collect();
        let mut calendar = vec![headers, vec![]];

        for _ in 0..day_of_week(0, month, year) + 1 {
            calendar
                .last_mut()
                .unwrap()
                .push((String::from("  "), false));
        }

        for d in 1..days_in_month(month, year) {
            let p = match d {
                0..=9 => format!("0{}", d),
                _ => d.to_string(),
            };

            let mut today = false;

            if d == day {
                today = true;
            }

            calendar.last_mut().unwrap().push((p, today));

            if day_of_week(d, month, year) == 6 {
                calendar.push(Vec::new());
            }
        }

        if calendar.last().unwrap().len() == 0 {
            calendar.pop();
        }

        let grid = self
            .builder
            .get_object::<gtk::Grid>("calendar_container")
            .unwrap();

        for (i, week) in calendar.iter().enumerate() {
            for (j, day) in week.iter().enumerate() {
                let label = Label::new(Some(&day.0));
                let aspect_frame = AspectFrame::new(None, 0.5, 0.5, 1.0, false);
                aspect_frame.add(&label);

                aspect_frame.get_style_context().add_class("flat");

                // If the day is today
                if day.1 {
                    // Add class calendar-today
                    label.get_style_context().add_class("calendar-today")
                }

                grid.attach(&aspect_frame, j as i32, i as i32, 1, 1);
            }
        }

        // Setup calendar popup
        // let open_calendar: Button = self.builder.get_object("open_calendar").unwrap();
        // self.calendar.set_pointing_to(Some(&open_calendar));
        // self.calendar.set_position(PositionType::Top);
    }

    fn add_interactions(&self) {
        let open_start_menu: Button = self.builder.get_object("open_start_menu").unwrap();
        let start_menu = self.start_menu.clone();

        open_start_menu.connect_clicked(move |_| start_menu.set_visible(!start_menu.is_visible()));

        let open_calendar: Button = self.builder.get_object("open_calendar").unwrap();
        let calendar = self.calendar.clone();

        open_calendar.connect_clicked(move |_| calendar.set_visible(!calendar.is_visible()));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let application = gtk::Application::new(Some("co.dothq.os.panel"), Default::default())?;

    application.connect_activate(|app| {
        // Load compiled resources
        let resources_bytes = include_bytes!("../resources/resources.gresource");
        let resource_data = glib::Bytes::from(&resources_bytes[..]);
        let res = gio::Resource::from_data(&resource_data).unwrap();
        gio::resources_register(&res);

        // Load application css
        let style = gtk::CssProvider::new();
        style.load_from_resource(&resource("panel.css"));
        StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().expect("Error initializing css provider"),
            &style,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let panel = Panel::new(&app);
        panel.pin();
        panel.add_interactions();
    });

    application.run(&args().collect::<Vec<_>>());

    unreachable!()
}
