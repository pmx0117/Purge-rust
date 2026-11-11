use super::*;
use std::sync::Arc;

impl PurgeView {
    pub(super) fn load_app_icons(&mut self, cx: &mut Context<Self>) {
        let load = cx.background_executor().spawn(async {
            let bitmaps = purge_engine::platform::application_icons();
            let paths = purge_engine::platform::application_icon_paths()
                .into_iter()
                .map(|(id, path)| {
                    let embedded = if path
                        .extension()
                        .is_some_and(|value| value.eq_ignore_ascii_case("icns"))
                    {
                        std::fs::read(&path).ok().and_then(|data| {
                            purge::application_image::icns_png(&data).map(Vec::from)
                        })
                    } else {
                        None
                    };
                    (id, path, embedded)
                })
                .collect::<Vec<_>>();
            (bitmaps, paths)
        });
        self._icon_task = Some(cx.spawn(async move |this, cx| {
            let (bitmaps, paths) = load.await;
            let mut icons: std::collections::HashMap<_, _> = bitmaps
                .into_iter()
                .filter_map(|(id, bitmap)| {
                    let buffer = image::RgbaImage::from_raw(bitmap.size, bitmap.size, bitmap.bgra)?;
                    Some((
                        id,
                        ImageSource::Render(Arc::new(RenderImage::new(vec![image::Frame::new(
                            buffer,
                        )]))),
                    ))
                })
                .collect();
            for (id, path, encoded) in paths {
                if let Some(png) = encoded {
                    icons.insert(
                        id,
                        ImageSource::Image(Arc::new(Image::from_bytes(ImageFormat::Png, png))),
                    );
                } else if path
                    .extension()
                    .is_none_or(|ext| !ext.eq_ignore_ascii_case("icns"))
                {
                    icons.insert(id, ImageSource::Resource(Resource::Path(path.into())));
                }
            }
            let _ = this.update(cx, |this, cx| {
                this.app_icons = icons;
                cx.notify();
            });
        }));
    }

    pub(super) fn application_icon(&self, item: &Item, side: f32) -> Div {
        let t = self.theme;
        let frame = div()
            .size(px(side))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center();
        if let Some(image) = self.app_icons.get(&item.id) {
            return frame.child(img(image.clone()).size(px(side)));
        }
        frame
            .rounded(px(side * 0.26))
            .bg(rgb(t.fill_hover))
            .border(px(0.5))
            .border_color(rgb(t.border))
            .child(icon("apps", t.muted, side * 0.55))
    }
}
