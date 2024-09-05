mod libsvg;
mod svg_view;

use crate::libsvg::{svg_process, Outline};
use crate::svg_view::SvgView;
use ori::prelude::*;
use rfd::AsyncFileDialog;
use std::path::PathBuf;

#[derive(Default)]
struct Data {
    pip: bool,
    outlines: Vec<Vec<Outline>>,
}

struct FileSelected {
    path: PathBuf,
}

struct AppDelegate;

impl Delegate<Data> for AppDelegate {
    fn event(&mut self, cx: &mut DelegateCx<Data>, data: &mut Data, event: &Event) -> bool {
        if let Some(file_selected) = event.cmd::<FileSelected>() {
            info!("File selected: {:?}", file_selected.path);

            let svg_data = std::fs::read_to_string(file_selected.path.clone()).unwrap();

            match svg_process(&svg_data, data.pip) {
                Ok(outlines) => {
                    data.outlines = outlines;
                }
                Err(err) => {
                    info!("No outlines found in SVG file: {:?}", err);
                }
            }

            return true;
        }

        false
    }
}

async fn pick_file(proxy: CommandProxy) {
    let path = AsyncFileDialog::new()
        .set_directory("/")
        .add_filter("SVG Files", &["svg"])
        .pick_file()
        .await;

    match path {
        Some(path) => {
            proxy.cmd(FileSelected {
                path: path.path().to_owned(),
            });
        }
        None => {
            info!("No file selected");
        }
    }
}

fn ui(data: &mut Data) -> impl View<Data> {
    let description = center(text!("SVG Splitter").font_size(32.0));
    let button = button(text("Pick SVG File"));
    let button = on_click(button, |cx, _| {
        let proxy = cx.proxy();
        cx.spawn_async(pick_file(proxy));
    });


    let pip_box = checkbox(data.pip);
    let pip_box = on_click(pip_box, |cx, data: &mut Data| {
        data.pip = !data.pip;
        cx.rebuild();
    });


    vstack![description, button, hstack![text!("Point in point check:"), pip_box].gap(5.0), flex(center(svgs(data)))]
}

fn svgs(data: &Data) -> impl View<Data> {
    let mut iter = data.outlines.iter().enumerate();
    let mut vview = vstack_vec().gap(10.0);
    let mut hview = hwrap_vec().gap(10.0);

    if let Some((i, _)) = iter.next() {
        vview.push(SvgView::new(i));
    }

    for (i, _) in iter {
        hview.push(SvgView::new(i));
    }

    vstack![vview, hview].gap(50.0)
}

fn main() {
    let window = Window::new().title("SVG Splitter");

    let app = App::build().window(window, ui).delegate(AppDelegate);
    let mut data = Data::default();

    ori::run(app, &mut data).unwrap();
}
