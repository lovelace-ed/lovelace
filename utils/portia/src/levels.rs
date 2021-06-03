/// Our interface is designed to "stack."
///
/// A very simple layout might look like this:
/// ```ignore
/// [item 1]
/// [item 2]
/// [item 3]
/// ```
///
/// A more complex layout might stack items not just vertically, but also horizontally.
/// ```ignore
/// [item 1a | item 1b | item 1c]
/// [   item 2a   |   item 2b   ]
/// ```
use malvolio::prelude::*;
use mercutio::*;

#[derive(Derivative, Debug, Clone)]
#[derivative(Default(new = "true"))]
/// A level of the layout heirachy.
pub struct Level {
    children: Vec<LevelChild>,
    layout_strategy: Option<LayoutStrategy>,
}

impl Level {
    /// Specify the strategy for this level.
    pub fn strategy<L>(mut self, strategy: L) -> Self
    where
        L: Into<LayoutStrategy>,
    {
        self.layout_strategy = Some(strategy.into());
        self
    }
    pub fn child<B>(mut self, child: B) -> Self
    where
        B: IntoLevelChild,
    {
        self.children.push(child.into_level_child());
        self
    }
    /// Add a series of children to this level, for example from an iterator.
    pub fn children<C, D>(mut self, children: C) -> Self
    where
        C: IntoIterator<Item = D>,
        D: IntoLevelChild,
    {
        self.children.extend(
            children
                .into_iter()
                .map(IntoLevelChild::into_level_child)
                .collect::<Vec<_>>(),
        );
        self
    }
    pub fn into_div(self) -> Div {
        self.into()
    }
}

impl From<Level> for Div {
    fn from(item: Level) -> Div {
        let layout_strategy = if let Some(layout_strategy) = item.layout_strategy {
            layout_strategy
        } else {
            LayoutStrategy {
                axis: LayoutAxis::Vertical,
                spacing: None,
            }
        };
        Div::new()
            .map(|div| match layout_strategy.axis {
                LayoutAxis::Horizontal => match layout_strategy.spacing {
                    Some(Spacing::Between) => div.apply(compose(
                        compose(FlexDirectionRow, DisplayFlex),
                        SpaceBetween,
                    )),
                    Some(Spacing::Fill) => {
                        div.apply(compose(compose(FlexDirectionRow, DisplayFlex), SpaceAround))
                    }
                    None => div.apply(FlexDirectionRow),
                },
                LayoutAxis::Vertical => match layout_strategy.spacing {
                    Some(Spacing::Between) => div.apply(compose(
                        compose(FlexDirectionColumn, DisplayFlex),
                        SpaceBetween,
                    )),
                    Some(Spacing::Fill) => div.apply(compose(
                        compose(FlexDirectionColumn, DisplayFlex),
                        SpaceAround,
                    )),
                    None => div.apply(FlexDirectionRow),
                },
            })
            .children(item.children.into_iter().map(|child| child.item))
    }
}

impl From<Level> for BodyNode {
    fn from(item: Level) -> BodyNode {
        let div: Div = From::from(item);
        div.into()
    }
}

#[derive(CSS)]
#[mercutio(css(display = "flex"), elements(Div))]
struct DisplayFlex;

#[derive(CSS)]
#[mercutio(css(flex_direction = "row"), elements(Div))]
struct FlexDirectionRow;

#[derive(CSS)]
#[mercutio(css(flex_direction = "column"), elements(Div))]
struct FlexDirectionColumn;

#[derive(CSS)]
#[mercutio(css(justify_content = "space-between"), elements(Div))]
struct SpaceBetween;

#[derive(CSS)]
#[mercutio(css(justify_content = "space-around"), elements(Div))]
struct SpaceAround;

#[derive(Copy, Clone, Debug, Derivative)]
#[derivative(Default(new = "true"))]
/// How to display the items – `Horizontal` means that items are laid out across a number of columns,
/// while `Vertical` means that they will be laid out in a number of rows (each item is below the
/// previous one).
pub enum LayoutAxis {
    #[derivative(Default)]
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Debug)]
pub enum Spacing {
    Between,
    Fill,
}

#[derive(Copy, Clone, Debug, Derivative)]
#[derivative(Default(new = "true"))]
/// Determines how to display the children of this level of the layout heirarchy.
pub struct LayoutStrategy {
    axis: LayoutAxis,
    spacing: Option<Spacing>,
}

impl LayoutStrategy {
    pub fn axis<A>(mut self, axis: A) -> Self
    where
        A: Into<LayoutAxis>,
    {
        self.axis = axis.into();
        self
    }
    pub fn spacing<S>(mut self, spacing: S) -> Self
    where
        S: Into<Spacing>,
    {
        self.spacing = Some(spacing.into());
        self
    }
}

#[derive(Clone, Debug)]
/// A child of a `Level`.
pub struct LevelChild {
    item: BodyNode,
    display_options: Option<DisplayOptions>,
}

impl LevelChild {
    /// Creates a new `LevelChild`
    pub fn new(item: BodyNode) -> Self {
        Self {
            item,
            display_options: None,
        }
    }
    /// Set the display options for this element.
    pub fn opts(mut self, options: DisplayOptions) -> Self {
        self.display_options = Some(options);
        self
    }
}

pub trait IntoLevelChild {
    fn into_level_child(self) -> LevelChild;
}

impl<T> IntoLevelChild for T
where
    T: Into<BodyNode>,
{
    fn into_level_child(self) -> LevelChild {
        LevelChild {
            item: self.into(),
            display_options: None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DisplayOptions {}
