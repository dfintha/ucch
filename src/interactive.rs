use crate::operations::OperationOptions;
use crate::result::Result;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::glib;
use gtk4::glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Align, Application, ApplicationWindow, Button, CssProvider, Grid, Image, Label,
    Scale,
};
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref OPTIONS: Mutex<OperationOptions> = Mutex::new(OperationOptions {
        tolerance: 0f64,
        crop_x: Some(0isize),
        crop_y: Some(0isize),
        crop_size: Some(0isize),
    });
}

pub(crate) fn perform_interactive_setup(file: &str) -> Result<OperationOptions> {
    let application = Application::builder()
        .application_id("org.dfintha.ucch")
        .build();

    let file = String::from(file);
    application.connect_activate(move |application| {
        let provider = CssProvider::new();
        provider.load_from_data(
            ".black_bg { background: black; } .note { color: gray; font-style: italic; }",
        );
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("failed to connect"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let window = ApplicationWindow::builder()
            .application(application)
            .title("UCCH")
            .modal(true)
            .resizable(false)
            .deletable(false)
            .build();

        let pixbuf = Pixbuf::from_file(&file).unwrap();
        let preview = Image::from_pixbuf(Some(&pixbuf));
        preview.set_css_classes(&["black_bg"]);

        let width = pixbuf.width();
        let height = pixbuf.height();
        let smaller = std::cmp::min(height, width);

        let crop_x_adjustment = Adjustment::builder()
            .lower(0f64)
            .upper((width - 1) as f64)
            .step_increment(1f64)
            .build();
        let crop_x_scale = Scale::builder()
            .adjustment(&crop_x_adjustment)
            .digits(0)
            .build();
        let crop_x_label = Label::builder().label("Crop X").halign(Align::End).build();
        let crop_x_value = Label::builder().label("0 px").halign(Align::Start).build();

        let crop_y_adjustment = Adjustment::builder()
            .lower(0f64)
            .upper((height - 1) as f64)
            .step_increment(1f64)
            .build();
        let crop_y_scale = Scale::builder()
            .adjustment(&crop_y_adjustment)
            .digits(0)
            .build();
        let crop_y_label = Label::builder().label("Crop Y").halign(Align::End).build();
        let crop_y_value = Label::builder().label("0 px").halign(Align::Start).build();

        let crop_size_adjustment = Adjustment::builder()
            .lower(1f64)
            .upper(smaller as f64)
            .build();
        let crop_size_scale = Scale::builder()
            .adjustment(&crop_size_adjustment)
            .digits(0)
            .build();
        let crop_size_label = Label::builder()
            .label("Crop Size")
            .halign(Align::End)
            .build();
        let crop_size_value = Label::builder().label("0 px²").halign(Align::Start).build();

        let tolerance_adjustment = Adjustment::builder()
            .lower(0f64)
            .upper(100f64)
            .step_increment(1f64)
            .build();
        let tolerance_scale = Scale::builder()
            .adjustment(&tolerance_adjustment)
            .digits(1)
            .build();
        let tolerance_label = Label::builder()
            .label("Tolerance")
            .halign(Align::End)
            .build();
        let tolerance_value = Label::builder().label("0%").halign(Align::Start).build();

        let note_label = Label::builder()
            .label("Note: The background erasure does not have a live preview.")
            .halign(Align::Center)
            .css_classes(["note"])
            .build();

        let ok = Button::builder().label("OK").build();

        let layout = Grid::builder()
            .column_homogeneous(true)
            .row_homogeneous(true)
            .column_spacing(10)
            .row_spacing(10)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        ok.connect_clicked(clone!(@weak window => move |_| window.close(); ));

        crop_x_scale.connect_value_changed(clone!(
            @weak crop_x_value,
            @weak crop_x_scale,
            @weak crop_y_scale,
            @weak crop_size_scale,
            @weak preview,
            @weak pixbuf => move |scale| {
                crop_x_value.set_label(&format!("{} px", scale.value() as u64));
                let size = crop_size_scale.value() as i32;
                let y = crop_y_scale.value() as i32;
                let max_x = pixbuf.width() - size - 1;
                let x = std::cmp::min(crop_x_scale.value() as i32, max_x);
                preview.set_from_pixbuf(Some(&pixbuf.new_subpixbuf(x, y, size, size)));
                scale.set_value(x as f64);
                OPTIONS.lock().unwrap().crop_x = Some(x as isize);
            }
        ));

        crop_y_scale.connect_value_changed(clone!(
            @weak crop_y_value,
            @weak crop_x_scale,
            @weak crop_y_scale,
            @weak crop_size_scale,
            @weak preview,
            @weak pixbuf => move |scale| {
                crop_y_value.set_label(&format!("{} px", scale.value() as u64));
                let size = crop_size_scale.value() as i32;
                let x = crop_x_scale.value() as i32;
                let max_y = pixbuf.height() - size - 1;
                let y = std::cmp::min(crop_y_scale.value() as i32, max_y);
                preview.set_from_pixbuf(Some(&pixbuf.new_subpixbuf(x, y, size, size)));
                scale.set_value(y as f64);
                OPTIONS.lock().unwrap().crop_y = Some(y as isize);
            }
        ));

        crop_size_scale.connect_value_changed(clone!(
            @weak crop_size_value,
            @weak crop_x_scale,
            @weak crop_y_scale,
            @weak crop_size_scale,
            @weak preview,
            @weak pixbuf => move |scale| {
                crop_size_value.set_label(&format!("{} px²", scale.value() as u64));
                let x = crop_x_scale.value() as i32;
                let y = crop_y_scale.value() as i32;
                let max_size_w = pixbuf.width() - x - 1;
                let max_size_h = pixbuf.height() - y - 1;
                let size = std::cmp::min(
                    std::cmp::min(max_size_w, max_size_h),
                    crop_size_scale.value() as i32
                );
                preview.set_from_pixbuf(Some(&pixbuf.new_subpixbuf(x, y, size, size)));
                scale.set_value(size as f64);
                OPTIONS.lock().unwrap().crop_size = Some(size as isize);
            }
        ));

        tolerance_scale.connect_value_changed(clone!(@weak tolerance_value => move |scale| {
            tolerance_value.set_label(&format!("{}%", scale.value() as u64));
            OPTIONS.lock().unwrap().tolerance = scale.value();
        }));

        crop_x_scale.set_value(((width - smaller) / 2) as f64);
        crop_y_scale.set_value(((height - smaller) / 2) as f64);
        crop_size_scale.set_value(smaller as f64);
        tolerance_scale.set_value(0f64);

        let preview_width = 4;
        let preview_height = 6;
        let scale_width = 3;

        let label_left = preview_width;
        let scale_left = preview_width + 1;
        let value_left = preview_width + scale_width + 1;

        layout.attach(&preview, 0, 0, preview_width, preview_height);

        layout.attach(&crop_x_label, label_left, 0, 1, 1);
        layout.attach(&crop_x_scale, scale_left, 0, scale_width, 1);
        layout.attach(&crop_x_value, value_left, 0, 1, 1);

        layout.attach(&crop_y_label, label_left, 1, 1, 1);
        layout.attach(&crop_y_scale, scale_left, 1, scale_width, 1);
        layout.attach(&crop_y_value, value_left, 1, 1, 1);

        layout.attach(&crop_size_label, label_left, 2, 1, 1);
        layout.attach(&crop_size_scale, scale_left, 2, scale_width, 1);
        layout.attach(&crop_size_value, value_left, 2, 1, 1);

        layout.attach(&tolerance_label, label_left, 3, 1, 1);
        layout.attach(&tolerance_scale, scale_left, 3, scale_width, 1);
        layout.attach(&tolerance_value, value_left, 3, 1, 1);

        layout.attach(&note_label, label_left, 4, scale_width + 2, 1);
        layout.attach(&ok, scale_left, 5, scale_width, 1);

        window.set_child(Some(&layout));
        window.show();
    });

    application.run_with_args(&Vec::<String>::new());
    Ok(*OPTIONS.lock().unwrap())
}
