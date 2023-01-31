use u8g2_fonts::{
    fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
    Error,
    Error::DisplayError,
    FontRenderer,
};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
};

#[derive(Clone)]
pub struct RoundedButton {
    pub btn: RoundedRectangle,
    pub text: String,
    pub font: FontRenderer,
    pub font_color: Rgb565,
    pub pressed_style: PrimitiveStyle<Rgb565>,
    pub released_style: PrimitiveStyle<Rgb565>,
    pub is_visible: bool,
}

#[allow(dead_code)]
impl RoundedButton {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn btn(mut self, rounded_button: RoundedRectangle) -> Self {
        self.btn = rounded_button;
        self
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = text.to_string();
        self
    }

    pub fn font(mut self, font: FontRenderer) -> Self {
        self.font = font;
        self
    }

    pub fn font_color(mut self, color: Rgb565) -> Self {
        self.font_color = color;
        self
    }

    pub fn pressed_style(mut self, pressed_style: PrimitiveStyle<Rgb565>) -> Self {
        self.pressed_style = pressed_style;
        self
    }

    pub fn released_style(mut self, released_style: PrimitiveStyle<Rgb565>) -> Self {
        self.released_style = released_style;
        self
    }

    pub fn is_visible(mut self, is_visible: bool) -> Self {
        self.is_visible = is_visible;
        self
    }

    pub fn show_pressed_button<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.btn
            .into_styled(self.pressed_style)
            .draw(display)
            .map_err(|e| DisplayError(e))?;
        self.render_button_text(display)?;

        Ok(())
    }

    pub fn show_released_button<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.btn
            .into_styled(self.released_style)
            .draw(display)
            .map_err(|e| DisplayError(e))?;
        self.render_button_text(display)?;

        Ok(())
    }

    fn render_button_text<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.font.render_aligned(
            self.text.as_str(),
            self.btn.bounding_box().center(),
            VerticalPosition::Center,
            HorizontalAlignment::Center,
            FontColor::Transparent(self.font_color),
            display,
        )?;

        Ok(())
    }
}

impl Default for RoundedButton {
    fn default() -> Self {
        Self {
            btn: RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(0, 0), Size::new(53, 25)),
                Size::new(5, 5),
            ),
            text: "text".to_string(),
            font: FontRenderer::new::<fonts::u8g2_font_9x15B_tr>(),
            font_color: Rgb565::BLACK,
            pressed_style: PrimitiveStyleBuilder::new()
                .stroke_width(2)
                .stroke_color(Rgb565::BLACK)
                .fill_color(Rgb565::CSS_DARK_GREEN)
                .build(),
            released_style: PrimitiveStyleBuilder::new()
                .stroke_width(2)
                .stroke_color(Rgb565::BLACK)
                .fill_color(Rgb565::CSS_LIGHT_GREEN)
                .build(),
            is_visible: true,
        }
    }
}

#[derive(Clone)]
pub struct Label {
    text: String,
    font: FontRenderer,
    font_color: Rgb565,
    background_color: Rgb565,
    position: Point,
    vertical_position: VerticalPosition,
    horizontal_alignment: HorizontalAlignment,
    text_bounding_box: Rectangle,
}

#[allow(dead_code)]
impl Label {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = text.to_string();
        self
    }

    pub fn font(mut self, font: FontRenderer) -> Self {
        self.font = font;
        self
    }

    pub fn font_color(mut self, color: Rgb565) -> Self {
        self.font_color = color;
        self
    }

    pub fn backgound(mut self, color: Rgb565) -> Self {
        self.background_color = color;
        self
    }

    pub fn position(mut self, position: Point) -> Self {
        self.position = position;
        self
    }

    pub fn vertical_position(mut self, v_position: VerticalPosition) -> Self {
        self.vertical_position = v_position;
        self
    }

    pub fn horizontal_alignment(mut self, h_alignment: HorizontalAlignment) -> Self {
        self.horizontal_alignment = h_alignment;
        self
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn show<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        match self.font.render_aligned(
            self.text.as_str(),
            self.position,
            self.vertical_position,
            self.horizontal_alignment,
            FontColor::Transparent(self.font_color),
            display,
        )? {
            Some(text_boinding_box) => self.text_bounding_box = text_boinding_box,
            None => self.text_bounding_box = Rectangle::zero(),
        };

        Ok(())
    }

    pub fn erase_old_text<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.text_bounding_box
            .into_styled(PrimitiveStyle::with_fill(self.background_color))
            .draw(display)
            .map_err(|e| DisplayError(e))?;

        Ok(())
    }
}

impl Default for Label {
    fn default() -> Self {
        Self {
            text: "text".to_string(),
            font: FontRenderer::new::<fonts::u8g2_font_9x15B_tr>(),
            font_color: Rgb565::BLACK,
            background_color: Rgb565::WHITE,
            position: Point::new(0, 0),
            vertical_position: VerticalPosition::Center,
            horizontal_alignment: HorizontalAlignment::Center,
            text_bounding_box: Rectangle::zero(),
        }
    }
}

pub struct Panel {
    panel: Rectangle,
    background: Rgb565,
    panel_labels: Vec<Label>,
    is_showing: bool,
}

impl Panel {
    pub fn new(top_left: Point, size: Size, background: Rgb565, panel_labels: Vec<Label>) -> Self {
        Self {
            panel: Rectangle::new(top_left, size),
            background,
            panel_labels,
            is_showing: false,
        }
    }

    pub fn show<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.is_showing = true;

        // Fill panel with the background color
        self.panel
            .into_styled(PrimitiveStyle::with_fill(self.background))
            .draw(display)
            .map_err(|e| DisplayError(e))?;

        // Show all the labels
        for label in &mut self.panel_labels {
            label.show(display)?;
        }

        Ok(())
    }

    pub fn hide<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.is_showing = false;

        self.panel
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(display)
            .map_err(|e| DisplayError(e))?;

        Ok(())
    }

    pub fn update_value<D>(
        &mut self,
        display: &mut D,
        label_id: usize,
        value: &str,
    ) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let the_label = &mut self.panel_labels[label_id];
        the_label.set_text(value);

        if self.is_showing {
            the_label.erase_old_text(display)?;
            the_label.show(display)?;
        }

        Ok(())
    }
}

pub struct NavigationPanel {
    panel: Rectangle,
    background: Rgb565,
    buttons: Vec<RoundedButton>,
    is_showing: bool,
}

impl NavigationPanel {
    pub fn new(
        top_left: Point,
        size: Size,
        background: Rgb565,
        buttons: Vec<RoundedButton>,
    ) -> Self {
        Self {
            panel: Rectangle::new(top_left, size),
            background,
            buttons,
            is_showing: false,
        }
    }

    pub fn show<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.is_showing = true;

        self.panel
            .into_styled(PrimitiveStyle::with_fill(self.background))
            .draw(display)
            .map_err(|e| DisplayError(e))?;

        for button in 0..self.buttons.len() {
            self.show_button_released(display, button)?;
        }

        Ok(())
    }

    pub fn show_button_released<D>(
        &mut self,
        display: &mut D,
        button_id: usize,
    ) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let this_button = &mut self.buttons[button_id];

        if this_button.is_visible {
            this_button.show_released_button(display)?;
        }

        Ok(())
    }

    pub fn show_button_pressed<D>(
        &mut self,
        display: &mut D,
        button_id: usize,
    ) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let this_button = &mut self.buttons[button_id];

        if this_button.is_visible {
            this_button.show_pressed_button(display)?;
        }

        Ok(())
    }

    pub fn hide<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.panel
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(display)
            .map_err(|e| DisplayError(e))?;

        Ok(())
    }
}
