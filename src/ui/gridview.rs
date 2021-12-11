use std::rc::Rc;

use druid::widget::ListIter;
use druid::{
  BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point,
  Size, UpdateCtx, Widget, WidgetPod,
};

pub struct GridViewItemCtx {
  pub index: usize,
}

#[derive(Data, Clone)]
pub struct GridViewState<T> {
  // __: std::marker::PhantomData<C>,
  pub items: T,
  pub x: usize,
}
pub struct GridView<T: Data> {
  children: Vec<WidgetPod<T, Box<dyn Widget<T>>>>,
  render_fn: Box<dyn Fn(&T, GridViewItemCtx) -> Box<dyn Widget<T>>>,
  n_drawable_children: usize,
  item_size: Size,
  spacing: f64,
  ncolumns: Rc<usize>,
}

impl<T: Data> GridView<T> {
  pub fn new<W: Widget<T> + 'static>(
    render_fn: impl Fn(&T, GridViewItemCtx) -> W + 'static,
  ) -> Self {
    GridView {
      children: vec![],
      render_fn: Box::new(move |data: &T, ctx: GridViewItemCtx| Box::new(render_fn(data, ctx))),
      n_drawable_children: 0,
      item_size: Size::new(10.0, 10.0),
      spacing: 8.,
      ncolumns: Rc::new(0),
    }
  }

  pub fn ncolumns(&self) -> usize {
    *self.ncolumns
  }

  pub fn with_item_size(mut self, cell_size: Size) -> Self {
    self.item_size = cell_size;
    self
  }

  pub fn with_spacing(mut self, spacing: f64) -> Self {
    self.spacing = spacing;
    self
  }

  /// When the widget is created or the data changes
  /// regenerate all child widgets
  fn update_child_count(&mut self, items: &impl ListIter<T>, _env: &Env) {
    let l = self.children.len();
    let l_new = items.data_len();
    if l > l_new {
      self.children.truncate(l_new);
    }
    items.for_each(|child_data, i| {
      let w = WidgetPod::new((self.render_fn)(child_data, GridViewItemCtx { index: i }));
      if i < l {
        self.children[i] = w;
      } else {
        self.children.push(w);
      }
    });
  }
}

impl<C: Data, T: ListIter<C>> Widget<GridViewState<T>> for GridView<C> {
  fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut GridViewState<T>, env: &Env) {
    let mut children = self.children.iter_mut();
    data.items.for_each_mut(|child_data, _| {
      if let Some(child) = children.next() {
        child.event(ctx, event, child_data, env);
      }
    });
  }

  fn lifecycle(
    &mut self,
    ctx: &mut LifeCycleCtx,
    event: &LifeCycle,
    data: &GridViewState<T>,
    env: &Env,
  ) {
    if let LifeCycle::WidgetAdded = event {
      self.update_child_count(&data.items, env);
      if data.items.data_len() > 0 {
        ctx.children_changed();
      }
    }

    let mut children = self.children.iter_mut();
    data.items.for_each(|child_data, _| {
      if let Some(child) = children.next() {
        child.lifecycle(ctx, event, child_data, env);
      }
    });
  }

  fn update(
    &mut self,
    ctx: &mut UpdateCtx,
    old_data: &GridViewState<T>,
    data: &GridViewState<T>,
    env: &Env,
  ) {
    let mut children = self.children.iter_mut();
    data.items.for_each(|child_data, _| {
      if let Some(child) = children.next() {
        child.update(ctx, child_data, env);
      }
    });

    if !old_data.same(data) {
      self.update_child_count(&data.items, env);
      ctx.children_changed();
    }
  }

  fn layout(
    &mut self,
    ctx: &mut LayoutCtx,
    bc: &BoxConstraints,
    data: &GridViewState<T>,
    env: &Env,
  ) -> Size {
    let nchildren = self.children.len() as f64;
    let greatest_width = (self.item_size.width + self.spacing) * nchildren;
    let width = greatest_width.min(bc.max().width).max(bc.min().width);
    let ncolumns =
      ((width - self.spacing) / (self.item_size.width + self.spacing)).floor() as usize;
    *Rc::get_mut(&mut self.ncolumns).unwrap() = ncolumns;

    let (height, nrows) = if ncolumns > 0 {
      let nrows = (nchildren / ncolumns as f64).ceil() as usize;
      let nrows_to_height =
        |nrows: usize| (nrows as f64) * (self.item_size.height + self.spacing) + self.spacing;
      let greatest_height = nrows_to_height(nrows);

      (greatest_height, nrows)
    } else {
      (bc.min().height, 0)
    };

    self.n_drawable_children = self.children.len().min(nrows * ncolumns);

    let mut x_pos = self.spacing;
    let mut y_pos = self.spacing;
    let mut idx = 0;
    let mut children = self.children.iter_mut();
    data.items.for_each(|child_data, _| {
      if idx < self.n_drawable_children {
        if let Some(child) = children.next() {
          child.layout(
            ctx,
            &BoxConstraints::new(self.item_size, self.item_size),
            child_data,
            env,
          );
          child.set_origin(ctx, child_data, env, Point::new(x_pos, y_pos));

          // Recur
          x_pos += self.item_size.width + self.spacing;
          // new row condition
          if (idx + 1) % ncolumns == 0 {
            x_pos = self.spacing;
            y_pos += self.item_size.height + self.spacing;
          }

          idx += 1;
        }
      }
    });

    Size { width, height }
  }

  fn paint(&mut self, ctx: &mut PaintCtx, data: &GridViewState<T>, env: &Env) {
    let mut children = self.children.iter_mut();
    data.items.for_each(|child_data, _| {
      if let Some(child) = children.next() {
        child.paint(ctx, child_data, env);
      }
    });
  }
}
