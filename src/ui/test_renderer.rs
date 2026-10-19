use egui::{ColorImage, FullOutput, TextureId};
use std::{collections::HashMap, io::Write, path::Path};

#[derive(Default)]
pub struct TestRenderer {
    textures: HashMap<TextureId, ColorImage>,
}
impl TestRenderer {
    pub fn update(&mut self, output: &FullOutput) {
        for (id, delta) in &output.textures_delta.set {
            let egui::ImageData::Color(image) = &delta.image;
            if let Some([x, y]) = delta.pos {
                let texture = self.textures.get_mut(id).expect("texture patch has a base");
                for row in 0..image.height() {
                    let start = (y + row) * texture.width() + x;
                    texture.pixels[start..start + image.width()].copy_from_slice(
                        &image.pixels[row * image.width()..(row + 1) * image.width()],
                    );
                }
            } else {
                self.textures.insert(*id, (**image).clone());
            }
        }
    }
    pub fn save(
        &self,
        ctx: &egui::Context,
        output: FullOutput,
        width: usize,
        height: usize,
        path: &Path,
    ) {
        let mut pixels = vec![[255u8; 3]; width * height];
        for clipped in ctx.tessellate(output.shapes, output.pixels_per_point) {
            let egui::epaint::Primitive::Mesh(mesh) = clipped.primitive else {
                continue;
            };
            let texture = self
                .textures
                .get(&mesh.texture_id)
                .expect("mesh has a texture");
            for triangle in mesh.indices.chunks_exact(3) {
                let [a, b, c] = [
                    mesh.vertices[triangle[0] as usize],
                    mesh.vertices[triangle[1] as usize],
                    mesh.vertices[triangle[2] as usize],
                ];
                let denominator = (b.pos.y - c.pos.y) * (a.pos.x - c.pos.x)
                    + (c.pos.x - b.pos.x) * (a.pos.y - c.pos.y);
                if denominator.abs() < 0.0001 {
                    continue;
                }
                let bounds = egui::Rect::from_min_max(
                    a.pos.min(b.pos).min(c.pos),
                    a.pos.max(b.pos).max(c.pos),
                )
                .intersect(clipped.clip_rect);
                let minx = bounds.min.x.floor().max(0.) as usize;
                let miny = bounds.min.y.floor().max(0.) as usize;
                let maxx = (bounds.max.x.ceil().max(0.) as usize).min(width);
                let maxy = (bounds.max.y.ceil().max(0.) as usize).min(height);
                for y in miny..maxy {
                    for x in minx..maxx {
                        let p = egui::pos2(x as f32 + 0.5, y as f32 + 0.5);
                        let u = ((b.pos.y - c.pos.y) * (p.x - c.pos.x)
                            + (c.pos.x - b.pos.x) * (p.y - c.pos.y))
                            / denominator;
                        let v = ((c.pos.y - a.pos.y) * (p.x - c.pos.x)
                            + (a.pos.x - c.pos.x) * (p.y - c.pos.y))
                            / denominator;
                        let w = 1. - u - v;
                        if u < 0. || v < 0. || w < 0. {
                            continue;
                        }
                        let uv = a.uv.to_vec2() * u + b.uv.to_vec2() * v + c.uv.to_vec2() * w;
                        let tx = (uv.x * texture.width() as f32)
                            .clamp(0., texture.width() as f32 - 1.)
                            as usize;
                        let ty = (uv.y * texture.height() as f32)
                            .clamp(0., texture.height() as f32 - 1.)
                            as usize;
                        let tex = texture[(tx, ty)].to_array();
                        let ca = a.color.to_array();
                        let cb = b.color.to_array();
                        let cc = c.color.to_array();
                        let mut color = [0.; 4];
                        for i in 0..4 {
                            color[i] = (f32::from(ca[i]) * u
                                + f32::from(cb[i]) * v
                                + f32::from(cc[i]) * w)
                                * f32::from(tex[i])
                                / 255.;
                        }
                        let alpha = color[3] / 255.;
                        for (i, channel) in pixels[y * width + x].iter_mut().enumerate() {
                            *channel = (color[i] + f32::from(*channel) * (1. - alpha))
                                .clamp(0., 255.) as u8;
                        }
                    }
                }
            }
        }
        let mut file = std::fs::File::create(path).expect("snapshot file");
        write!(file, "P6\n{width} {height}\n255\n").expect("snapshot header");
        let bytes: Vec<_> = pixels.into_iter().flatten().collect();
        file.write_all(&bytes).expect("snapshot pixels");
    }
}

pub fn text_rects(output: &FullOutput) -> Vec<(String, egui::Rect)> {
    fn visit(shape: &egui::Shape, result: &mut Vec<(String, egui::Rect)>) {
        match shape {
            egui::Shape::Text(text) => result.push((
                text.galley.job.text.clone(),
                egui::Rect::from_min_size(text.pos, text.galley.size()),
            )),
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    visit(shape, result);
                }
            }
            _ => {}
        }
    }
    let mut result = Vec::new();
    for clipped in &output.shapes {
        visit(&clipped.shape, &mut result);
    }
    result
}
