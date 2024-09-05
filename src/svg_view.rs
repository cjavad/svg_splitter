use crate::Data;
use ori::prelude::*;

const FLIP_Y: Vector = Vector::new(1.0, -1.0);

#[derive(Build)]
pub struct SvgView {
    outline_idx: usize,
}

#[doc(hidden)]
pub struct SvgViewState {
    bounds: Rect,
    canvas: Canvas,
}

impl SvgView {
    pub fn new(outline_idx: usize) -> Self {
        Self { outline_idx }
    }
}

impl View<Data> for SvgView {
    type State = SvgViewState;

    fn build(&mut self, cx: &mut BuildCx, data: &mut Data) -> Self::State {
        let mut canvas = Canvas::new();

        let bounds = self.draw_svg(data, &mut canvas);

        SvgViewState { canvas, bounds }
    }

    fn rebuild(
        &mut self,
        state: &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut Data,
        old: &Self,
    ) {
        state.canvas.clear();

        state.bounds = self.draw_svg(data, &mut state.canvas);

        cx.draw();
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut Data, event: &Event) {}

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut Data,
        space: Space,
    ) -> Size {
        space.fit(state.bounds.size())
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut Data) {
        let scale = cx.size() / state.bounds.size();

        let mut affine = Affine::IDENTITY;

        affine *= Affine::scale(scale.to_vector());
        affine *= Affine::translate(-state.bounds.offset());

        cx.transformed(affine, |cx| {
            cx.draw_canvas(state.canvas.clone());
            cx.stroke(Curve::rect(state.bounds), Stroke::from(1.0), Color::RED);
        });
    }
}

impl SvgView {
    fn draw_svg(&self, data: &Data, canvas: &mut Canvas) -> Rect {
        let mut curve = Curve::new();

        if self.outline_idx >= data.outlines.len() {
            return Rect::ZERO;
        }

        for outline in &data.outlines[self.outline_idx] {
            let mut vertices = outline.vertices.iter();

            match vertices.next() {
                Some(vertex) => curve.move_to(*vertex * FLIP_Y),
                None => continue,
            }

            for vertex in vertices {
                curve.line_to(*vertex * FLIP_Y);
            }

            curve.close();
        }

        let bounds = curve.bounds();

        canvas.fill(curve, FillRule::EvenOdd, Color::WHITE);

        bounds
    }
}
