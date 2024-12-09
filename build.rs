use ico::{IconDirEntry, IconImage, ResourceType};
use resvg::{
    tiny_skia::Pixmap,
    usvg::{Options, Transform, Tree},
};
use std::{fs::File, io::Read};

#[allow(dead_code)]
enum FileType {
    Png,
    Ico,
}

fn main() {
    println!("cargo::rerun-if-changed=public/logo.svg");
    let mut logofile = File::open("public/logo.svg").unwrap();

    let mut logofile_data = String::new();
    logofile.read_to_string(&mut logofile_data).unwrap();
    let logo_svg = Tree::from_str(&logofile_data, &Options::default()).unwrap();

    make_logo_at_size(&logo_svg, 32.0, FileType::Ico, "public/favicon.ico");
}

fn make_logo_at_size(data: &Tree, size: f32, ty: FileType, output_path: &str) {
    let mut pixmap = Pixmap::new(size as u32, size as u32).unwrap();

    resvg::render(
        data,
        Transform::from_translate(0.0, 0.0).post_scale(
            size / data.root().abs_bounding_box().width(),
            size / data.root().abs_bounding_box().height(),
        ),
        &mut pixmap.as_mut(),
    );

    match ty {
        FileType::Png => {
            pixmap.save_png(output_path).unwrap();
        }
        FileType::Ico => {
            let outfile = File::create(output_path).unwrap();
            let png = pixmap.encode_png().unwrap();
            let mut ico = ico::IconDir::new(ResourceType::Icon);
            ico.add_entry(
                IconDirEntry::encode(&IconImage::read_png(png.as_slice()).unwrap()).unwrap(),
            );

            ico.write(&outfile).unwrap();
        }
    }
}
