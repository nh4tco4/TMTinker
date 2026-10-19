use crate::{
    core::{alphabet::ValidatedAlphabet, tape::Tape},
    diagram::{
        compile::compile,
        model::{
            document::DiagramDocument,
            layout::LayoutPos,
            node::{Direction, NodeAction},
            transition::TransitionSymbols,
        },
        runtime::DiagramRunSession,
    },
};
use egui::{Color32, Pos2, Rect, TextureHandle, TextureOptions, Vec2};

pub struct MenuBackground {
    session: DiagramRunSession,
    texture: Option<TextureHandle>,
    next_step: f64,
}
impl Default for MenuBackground {
    fn default() -> Self {
        let alphabet = ValidatedAlphabet::new([' ', '0', '1']).expect("static demo alphabet");
        let mut document = DiagramDocument::new(alphabet.clone());
        let key = document.entry_submachine();
        let machine = document.submachine_mut(key).expect("demo entry");
        let actions = [
            NodeAction::Start,
            NodeAction::Write('0'),
            NodeAction::Move {
                direction: Direction::Right,
                repeat: std::num::NonZeroU32::MIN,
            },
            NodeAction::Write('1'),
            NodeAction::Move {
                direction: Direction::Left,
                repeat: std::num::NonZeroU32::MIN,
            },
        ];
        let nodes: Vec<_> = actions
            .into_iter()
            .enumerate()
            .map(|(i, action)| machine.add_node(LayoutPos::new(i as f32 * 75., 0.), action))
            .collect();
        for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 1)] {
            machine.add_edge(
                TransitionSymbols::new(alphabet.iter().copied(), &alphabet)
                    .expect("demo transitions"),
                nodes[a],
                nodes[b],
            );
        }
        Self {
            session: DiagramRunSession::new(compile(&document).expect("valid demo"), Tape::new()),
            texture: None,
            next_step: 0.,
        }
    }
}
impl MenuBackground {
    pub fn show(&mut self, ctx: &egui::Context, rect: Rect) {
        let now = ctx.input(|i| i.time);
        if self.texture.is_none() || now >= self.next_step {
            self.session.step();
            self.next_step = now + 0.85;
            let source = self.session.compiled().source();
            let machine = source
                .submachine(source.entry_submachine())
                .expect("private demo exists");
            let active = machine
                .machine()
                .nodes_iter()
                .position(|(key, _)| key == self.session.source_location().node)
                .unwrap_or(0);
            let image = demo_image(active, self.session.tape().head_position());
            if let Some(texture) = &mut self.texture {
                texture.set(image, TextureOptions::LINEAR);
            } else {
                self.texture =
                    Some(ctx.load_texture("menu_demo_blurred", image, TextureOptions::LINEAR));
            }
        }
        if let Some(texture) = &self.texture {
            let painter = ctx.layer_painter(egui::LayerId::background());
            painter.image(
                texture.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
                Color32::WHITE,
            );
            painter.rect_filled(rect, 0., Color32::from_white_alpha(24));
        }
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(
            (self.next_step - now).max(0.01),
        ));
    }
}

fn demo_image(active: usize, head: i32) -> egui::ColorImage {
    const W: usize = 280;
    const H: usize = 180;
    let centers = [
        Pos2::new(68., 98.),
        Pos2::new(170., 98.),
        Pos2::new(280., 98.),
        Pos2::new(390., 98.),
        Pos2::new(450., 194.),
    ];
    let mut pixels = vec![Color32::from_rgb(240, 244, 246); W * H];
    for y in 0..H {
        for x in 0..W {
            let p = Pos2::new(x as f32 * 2., y as f32 * 2.);
            let mut color = pixels[y * W + x];
            for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 1)] {
                let start = centers[a];
                let end = centers[b];
                let v = end - start;
                let t = ((p - start).dot(v) / v.length_sq()).clamp(0., 1.);
                if p.distance(start + v * t) < 1.4 {
                    color = Color32::from_rgb(110, 145, 150);
                }
            }
            for (i, center) in centers.iter().enumerate() {
                let distance = p.distance(*center);
                if distance < 27. {
                    color = if i == active {
                        Color32::from_rgb(67, 185, 171)
                    } else {
                        Color32::from_rgb(224, 230, 232)
                    };
                }
                if (distance - 27.).abs() < 1.3 {
                    color = Color32::from_rgb(136, 159, 163);
                }
            }
            for cell in 0..11 {
                let r =
                    Rect::from_min_size(Pos2::new(48. + cell as f32 * 43., 270.), Vec2::splat(38.));
                if r.contains(p) {
                    color = if cell == 5 + head {
                        Color32::from_rgb(67, 185, 171)
                    } else {
                        Color32::from_rgb(225, 232, 234)
                    };
                    if (p.x - r.left())
                        .min(r.right() - p.x)
                        .min((p.y - r.top()).min(r.bottom() - p.y))
                        < 1.4
                    {
                        color = Color32::from_rgb(170, 185, 190);
                    }
                }
            }
            pixels[y * W + x] = color;
        }
    }

    for _ in 0..3 {
        for horizontal in [true, false] {
            let previous = pixels.clone();
            let (lines, length) = if horizontal { (H, W) } else { (W, H) };
            for line in 0..lines {
                let sample = |offset: i32| {
                    let offset = offset.clamp(0, length as i32 - 1) as usize;
                    previous[if horizontal {
                        line * W + offset
                    } else {
                        offset * W + line
                    }]
                };
                let mut sum = [0i32; 3];
                for offset in -3..=3 {
                    let c = sample(offset);
                    sum[0] += i32::from(c.r());
                    sum[1] += i32::from(c.g());
                    sum[2] += i32::from(c.b());
                }
                for offset in 0..length {
                    let index = if horizontal {
                        line * W + offset
                    } else {
                        offset * W + line
                    };
                    pixels[index] = Color32::from_rgb(
                        (sum[0] / 7) as u8,
                        (sum[1] / 7) as u8,
                        (sum[2] / 7) as u8,
                    );
                    let old = sample(offset as i32 - 3);
                    let new = sample(offset as i32 + 4);
                    sum[0] += i32::from(new.r()) - i32::from(old.r());
                    sum[1] += i32::from(new.g()) - i32::from(old.g());
                    sum[2] += i32::from(new.b()) - i32::from(old.b());
                }
            }
        }
    }
    egui::ColorImage::new([W, H], pixels)
}
