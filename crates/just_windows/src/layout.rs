use just_x11::{requests::ConfigureWindowAttributes, Rectangle, WindowId};

#[derive(Debug, Clone, Copy)]
pub struct PositionedWindow {
    pub window: WindowId,
    pub position: Rectangle,
    pub border_width: u16,
    pub border_color: u32,
}

impl PositionedWindow {
    pub fn to_attributes(self) -> ConfigureWindowAttributes {
        ConfigureWindowAttributes::new()
            .set_width(self.position.width as u16)
            .set_height(self.position.height as u16)
            .set_x(self.position.x as i16)
            .set_y(self.position.y as i16)
            .set_border_width(self.border_width)
    }
}

pub trait Layout {
    fn position_windows(
        &self,
        area: Rectangle,
        active_window: Option<WindowId>,
        windows: &[WindowId],
    ) -> Vec<PositionedWindow>;
}

pub struct SingleWindow {
    pub border_width: u16,
    pub window_pad: u16,
    pub active_border: u32,
    pub inactive_border: u32,
}

impl Layout for SingleWindow {
    fn position_windows(
        &self,
        area: Rectangle,
        active_window: Option<WindowId>,
        windows: &[WindowId],
    ) -> Vec<PositionedWindow> {
        if let Some(&window) = windows.get(0) {
            let master_positioned = {
                let border_color = if active_window == Some(window) {
                    self.active_border
                } else {
                    self.inactive_border
                };

                let width = area.width - self.border_width * 2 - self.window_pad * 2;
                let height = (area.height - (self.window_pad * 2)) - self.border_width * 2;
                let x = self.window_pad as i16 + area.x;
                let y = self.window_pad as i16 + area.y;

                PositionedWindow {
                    window,
                    position: Rectangle {
                        width,
                        height,
                        x,
                        y,
                    },
                    border_width: self.border_width,
                    border_color,
                }
            };
            vec![master_positioned]
        } else {
            Vec::new()
        }
    }
}

/// Vertical screen split with master window on the left and rest on the right.
pub struct VerticalMasterSplit {
    pub border_width: u16,
    pub window_pad: u16,
    pub active_border: u32,
    pub inactive_border: u32,
    pub right: Box<dyn Layout>,
}

impl Layout for VerticalMasterSplit {
    fn position_windows(
        &self,
        area: Rectangle,
        active_window: Option<WindowId>,
        windows: &[WindowId],
    ) -> Vec<PositionedWindow> {
        if let Some((&master_window, rest_windows)) = windows.split_first() {
            if rest_windows.is_empty() {
                SingleWindow {
                    border_width: self.border_width,
                    window_pad: self.window_pad,
                    active_border: self.active_border,
                    inactive_border: self.inactive_border,
                }
                .position_windows(area, active_window, &[master_window])
            } else {
                let left = SingleWindow {
                    border_width: self.border_width,
                    window_pad: self.window_pad,
                    active_border: self.active_border,
                    inactive_border: self.inactive_border,
                }
                .position_windows(
                    Rectangle {
                        x: area.x,
                        y: area.y,
                        width: area.width / 2 + self.window_pad / 2,
                        height: area.height,
                    },
                    active_window,
                    &[master_window],
                );

                let right = self.right.position_windows(
                    Rectangle {
                        x: area.x + (area.width as i16 / 2 - self.window_pad as i16 / 2),
                        y: area.y,
                        width: area.width / 2 + self.window_pad / 2,
                        height: area.height,
                    },
                    active_window,
                    rest_windows,
                );

                let mut combined = Vec::with_capacity(left.len() + right.len());
                combined.extend(left);
                combined.extend(right);

                combined
            }
        } else {
            vec![]
        }
    }
}

#[derive(Clone)]
pub struct VerticalStack {
    pub border_width: u16,
    pub window_pad: u16,
    pub active_border: u32,
    pub inactive_border: u32,
}

impl Layout for VerticalStack {
    /// Arrange windows in a vertical stack
    #[must_use]
    fn position_windows(
        &self,
        area: Rectangle,
        active_window: Option<WindowId>,
        windows: &[WindowId],
    ) -> Vec<PositionedWindow> {
        let window_count = windows.len() as u16;

        let x = self.window_pad as i16 + area.x;
        let width = area.width - self.border_width * 2 - self.window_pad * 2;
        let height = (area.height - (self.window_pad * (window_count + 1))) / window_count
            - self.border_width * 2;

        let ret = windows
            .iter()
            .enumerate()
            .map(|(idx, &window)| {
                let border_color = if active_window == Some(window) {
                    self.active_border
                } else {
                    self.inactive_border
                };

                let y = (self.window_pad as i16 * 1 + area.y)
                    + idx as i16
                        * (height as i16 + 2 * self.border_width as i16 + self.window_pad as i16);

                let effective_height = if idx + 1 == window_count as usize {
                    // We lose some space due to integer division so if we're at the last (bottom)
                    // window we override height to take all remaining space. The difference in
                    // height is invisible (by me) until you stack more than 10 windows which
                    // is unlikely to happan in real use scenario.
                    (area.height as i16 - y + area.y
                        - self.border_width as i16 * 2
                        - self.window_pad as i16) as u16
                } else {
                    height
                };

                let ret = PositionedWindow {
                    window,
                    position: Rectangle {
                        x,
                        y,
                        width,
                        height: effective_height,
                    },
                    border_color,
                    border_width: self.border_width,
                };

                ret
            })
            .collect();

        ret
    }
}
