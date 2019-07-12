// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::skia;
use usvg::{try_opt, try_opt_warn};

use crate::{prelude::*, backend_utils::*};


pub fn draw(
    image: &usvg::Image,
    opt: &Options,
    canvas: &mut skia::Canvas,
) -> Rect {
    if image.visibility != usvg::Visibility::Visible {
        return image.view_box.rect;
    }

    if image.format == usvg::ImageFormat::SVG {
        draw_svg(&image.data, image.view_box, opt, canvas);
    } else {
        draw_raster(&image.data, image.view_box, image.rendering_mode, opt, canvas);
    }

    image.view_box.rect
}

pub fn draw_raster(
    data: &usvg::ImageData,
    view_box: usvg::ViewBox,
    _rendering_mode: usvg::ImageRendering,
    opt: &Options,
    canvas: &mut skia::Canvas,
) {
    let img = match data {
        usvg::ImageData::Path(ref path) => {
            let path = image::get_abs_path(path, opt);
            try_opt_warn!(
                skia::Surface::from_file(&path),
                "Failed to load an external image: {:?}.", path
            )
        }
        usvg::ImageData::Raw(ref data) => {
            try_opt_warn!(
                skia::Surface::from_data(data),
                "Failed to load an embedded image."
            )
        }
    };

    let img_size = try_opt!(ScreenSize::new(img.get_width() as u32, img.get_height() as u32));

    canvas.save();
    // TODO:  This is filter quality I assume?
    // if rendering_mode == usvg::ImageRendering::OptimizeSpeed {
    //     p.set_smooth_pixmap_transform(false);
    // }

    if view_box.aspect.slice {
        let r = view_box.rect;
        canvas.clip_rect(r.x(), r.y(), r.width(), r.height());
    }

    let r = image::image_rect(&view_box, img_size);
    canvas.draw_surface_rect(&img, r.x(), r.y(), r.width(), r.height());

    // // Revert.
    // p.set_smooth_pixmap_transform(true);
    canvas.restore();


}

pub fn draw_svg(
    data: &usvg::ImageData,
    view_box: usvg::ViewBox,
    opt: &Options,
    canvas: &mut skia::Canvas,
) {
    let (tree, sub_opt) = try_opt!(image::load_sub_svg(data, opt));

    let img_size = tree.svg_node().size.to_screen_size();
    let (ts, clip) = image::prepare_sub_svg_geom(view_box, img_size);

    canvas.save();

    if let Some(clip) = clip {
        canvas.clip_rect(clip.x(), clip.y(), clip.width(), clip.height());
    }

    canvas.concat(&ts.to_native());
    super::render_to_canvas(&tree, &sub_opt, img_size, canvas);
    
    canvas.restore();
}
