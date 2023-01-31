use u8g2_fonts::{fonts, Error, FontRenderer};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Rectangle, RoundedRectangle},
};

use crate::gui::widgets::{Label, Panel, RoundedButton, NavigationPanel};

const FONT_LUBS12: FontRenderer = FontRenderer::new::<fonts::u8g2_font_luBS12_tr>();
const FONT_LUBS24: FontRenderer = FontRenderer::new::<fonts::u8g2_font_luBS24_tr>();

#[derive(Clone, Copy)]
pub enum ViewId {
    IndoorOutdoor,
    WindRainStatus,
    TimeDate,
}

pub struct Views {
    pub indoor_outdoor_view: IndoorOutdoorView,
    pub wind_rain_status_view: WindRainStatusView,
    pub time_date_view: TimeDateView,
}

impl Views {
    pub fn build_views() -> Self {
        Self {
            indoor_outdoor_view: IndoorOutdoorView::build(),
            wind_rain_status_view: WindRainStatusView::build(),
            time_date_view: TimeDateView::bulid(),
        }
    }

    pub fn show_view<D>(&mut self, display: &mut D, view_id: ViewId) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        match view_id {
            ViewId::IndoorOutdoor => self.indoor_outdoor_view.show(display)?,
            ViewId::WindRainStatus => self.wind_rain_status_view.show(display)?,
            ViewId::TimeDate => self.time_date_view.show(display)?,
        }

        Ok(())
    }

    pub fn hide_view<D>(&mut self, display: &mut D, view_id: ViewId) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        match view_id {
            ViewId::IndoorOutdoor => self.indoor_outdoor_view.hide(display)?,
            ViewId::WindRainStatus => self.wind_rain_status_view.hide(display)?,
            ViewId::TimeDate => self.time_date_view.hide(display)?,
        }

        Ok(())
    }
}

pub struct IndoorOutdoorView {
    pub indoor_panel: Panel,
    pub outdoor_panel: Panel,
    pub navigation_panel: NavigationPanel,
}

impl IndoorOutdoorView {
    pub fn build() -> Self {
        Self {
            indoor_panel: IndoorPanel::build(
                Point::new(0, 0),
                Size::new(320, 80),
                Rgb565::CSS_LIGHT_GREEN,
            ),
            outdoor_panel: OutdoorPanel::build(
                Point::new(0, 82),
                Size::new(320, 117),
                Rgb565::CSS_LIGHT_GREEN,
            ),
            navigation_panel: NavPanel::build(
                Point::new(0, 201),
                Size::new(320, 39),
                Rgb565::CSS_DARK_SLATE_BLUE,
                false,
            ),
        }
    }

    pub fn show<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.indoor_panel.show(display)?;
        self.outdoor_panel.show(display)?;
        self.navigation_panel.show(display)?;

        Ok(())
    }

    pub fn hide<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.indoor_panel.hide(display)?;
        self.outdoor_panel.hide(display)?;
        self.navigation_panel.hide(display)?;

        Ok(())
    }
}

pub struct WindRainStatusView {
    pub wind_panel: Panel,
    pub rain_panel: Panel,
    pub status_panel: Panel,
    pub navigation_panel: NavigationPanel,
}

impl WindRainStatusView {
    pub fn build() -> Self {
        Self {
            wind_panel: WindPanel::build(
                Point::new(0, 0),
                Size::new(320, 65),
                Rgb565::CSS_DARK_KHAKI,
            ),
            rain_panel: RainPanel::build(
                Point::new(0, 67),
                Size::new(320, 65),
                Rgb565::CSS_DARK_KHAKI,
            ),
            status_panel: StatusPanel::build(
                Point::new(0, 134),
                Size::new(320, 65),
                Rgb565::CSS_DARK_KHAKI,
            ),
            navigation_panel: NavPanel::build(
                Point::new(0, 201),
                Size::new(320, 39),
                Rgb565::CSS_DARK_SLATE_BLUE,
                false,
            ),
        }
    }

    pub fn show<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.wind_panel.show(display)?;
        self.rain_panel.show(display)?;
        self.status_panel.show(display)?;
        self.navigation_panel.show(display)?;

        Ok(())
    }

    pub fn hide<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.wind_panel.hide(display)?;
        self.rain_panel.hide(display)?;
        self.status_panel.hide(display)?;
        self.navigation_panel.hide(display)?;

        Ok(())
    }
}

pub struct TimeDateView {
    pub time_date_panel: Panel,
    pub navigation_panel: NavigationPanel,
}

impl TimeDateView {
    pub fn bulid() -> Self {
        Self {
            time_date_panel: TimeDatePanel::build(
                Point::new(0, 0),
                Size::new(320, 200),
                Rgb565::BLACK,
            ),
            navigation_panel: NavPanel::build(
                Point::new(0, 201),
                Size::new(320, 39),
                Rgb565::CSS_DARK_SLATE_BLUE,
                true,
            ),
        }
    }

    pub fn show<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.time_date_panel.show(display)?;
        self.navigation_panel.show(display)?;

        Ok(())
    }

    pub fn hide<D>(&mut self, display: &mut D) -> Result<(), Error<D::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.time_date_panel.hide(display)?;
        self.navigation_panel.hide(display)?;

        Ok(())
    }
}

pub enum WindValueId {
    CurrentWindSpeed = 2,
    AverageWindSpeed = 4,
    PeakWindSpeed = 6,
}

struct WindPanel {}

/**
 * Build wind panel
 *
 *           Panel Labels Vector Layout
 *        Element             Element Name
 *  --------------------------------------------------
 *          0           Header
 *          1           Current WindSpeed title
 *          2           Current WindSpeed value
 *          3           Average WindSpeed title
 *          4           Average WindSpeed value
 *          5           Peak WindSpeed title
 *          6           Peak WindSpeed value
 */
impl WindPanel {
    pub fn build(top_left: Point, size: Size, background: Rgb565) -> Panel {
        let mut panel_labels = Vec::with_capacity(7);

        // Header
        let header_label = Label::new()
            .text("Wind Speed")
            .font(FONT_LUBS12)
            .font_color(Rgb565::CSS_DARK_RED)
            .backgound(background)
            .position(top_left + Point::new(160, 16));
        panel_labels.push(header_label);

        // Title - Current WindSpeed
        let mut title_label = Label::new()
            .text("Current")
            .position(top_left + Point::new(55, 36));
        panel_labels.push(title_label.clone());

        // Value - Current WinndSpeed
        let mut value_label = Label::new()
            .text("--")
            .font_color(Rgb565::BLUE)
            .backgound(background)
            .position(top_left + Point::new(55, 52));
        panel_labels.push(value_label.clone());

        // Title - Average WindSpeed
        title_label = title_label
            .text("Average")
            .position(top_left + Point::new(160, 36));
        panel_labels.push(title_label.clone());

        // Value - Average WindSpeed
        value_label = value_label
            .text("--")
            .position(top_left + Point::new(160, 52));
        panel_labels.push(value_label.clone());

        // Title - Peak WindSpeed
        title_label = title_label
            .text("Peak")
            .position(top_left + Point::new(270, 36));
        panel_labels.push(title_label);

        // Value - Peak WindSpeed
        value_label = value_label
            .text("--")
            .position(top_left + Point::new(270, 52));
        panel_labels.push(value_label);

        Panel::new(top_left, size, background, panel_labels)
    }
}

pub enum RainValueId {
    DailyRainfall = 2,
    MonthlyRainfall = 4,
    YearlyRainfall = 6,
}
struct RainPanel {}

/**
 * Build rain panel
 *
 *           Panel Labels Vector Layout
 *        Element             Element Name
 *  --------------------------------------------------
 *          0           Header
 *          1           Daily Rainfall title
 *          2           Daily Rainfall value
 *          3           Monthly Rainfall title
 *          4           Monthly Rainfall value
 *          5           Yearly Rainfall title
 *          6           Yearly Rainfall value
 */

impl RainPanel {
    pub fn build(top_left: Point, size: Size, background: Rgb565) -> Panel {
        let mut panel_labels = Vec::with_capacity(7);

        let header_label = Label::new()
            .text("Rainfall")
            .font(FONT_LUBS12)
            .font_color(Rgb565::CSS_DARK_RED)
            .backgound(background)
            .position(top_left + Point::new(160, 16));
        panel_labels.push(header_label);

        let mut title_label = Label::new()
            .text("Daily")
            .position(top_left + Point::new(55, 36));
        panel_labels.push(title_label.clone());

        let mut value_label = Label::new()
            .text("--")
            .font_color(Rgb565::BLUE)
            .backgound(background)
            .position(top_left + Point::new(55, 52));
        panel_labels.push(value_label.clone());

        title_label = title_label
            .text("Monthly")
            .position(top_left + Point::new(160, 36));
        panel_labels.push(title_label.clone());

        value_label = value_label
            .text("--")
            .position(top_left + Point::new(160, 52));
        panel_labels.push(value_label.clone());

        title_label = title_label
            .text("Yearly")
            .position(top_left + Point::new(270, 36));
        panel_labels.push(title_label);

        value_label = value_label
            .text("--")
            .position(top_left + Point::new(270, 52));
        panel_labels.push(value_label);

        Panel::new(top_left, size, background, panel_labels)
    }
}

pub enum StatusValueId {
    Battery = 2,
    Channel = 4,
    ProductId = 6,
}
struct StatusPanel {}

/**
 * Build status panel
 *
 *           Panel Labels Vector Layout
 *        Element             Element Name
 *  --------------------------------------------------
 *          0           Header
 *          1           Battery title
 *          2           Battery value
 *          3           Channel title
 *          4           Channel value
 *          5           Product title
 *          6           Prodcut value
 */

impl StatusPanel {
    pub fn build(top_left: Point, size: Size, background: Rgb565) -> Panel {
        let mut panel_labels = Vec::with_capacity(7);

        let header_label = Label::new()
            .text("Status")
            .font(FONT_LUBS12)
            .font_color(Rgb565::CSS_DARK_RED)
            .backgound(background)
            .position(top_left + Point::new(160, 16));
        panel_labels.push(header_label);

        let mut title_label = Label::new()
            .text("Battery")
            .position(top_left + Point::new(55, 36));
        panel_labels.push(title_label.clone());

        let mut value_label = Label::new()
            .text("--")
            .font_color(Rgb565::BLUE)
            .backgound(background)
            .position(top_left + Point::new(55, 52));
        panel_labels.push(value_label.clone());

        title_label = title_label
            .text("Channel")
            .position(top_left + Point::new(160, 36));
        panel_labels.push(title_label.clone());

        value_label = value_label
            .text("--")
            .position(top_left + Point::new(160, 52));
        panel_labels.push(value_label.clone());

        title_label = title_label
            .text("Product")
            .position(top_left + Point::new(270, 36));
        panel_labels.push(title_label);

        value_label = value_label
            .text("--")
            .position(top_left + Point::new(270, 52));
        panel_labels.push(value_label);

        Panel::new(top_left, size, background, panel_labels)
    }
}

pub enum IndoorValueId {
    Pressure = 2,
    Temperature = 4,
    Humidity = 6,
}
struct IndoorPanel {}

/**
 * Build indoor panel
 *
 *           Panel Labels Vector Layout
 *        Element             Element Name
 *  --------------------------------------------------
 *          0           Header
 *          1           Pressure title
 *          2           Pressure value
 *          3           Temperature title
 *          4           Temperature  value
 *          5           Humidity title
 *          6           Humidity value
 */

impl IndoorPanel {
    pub fn build(top_left: Point, size: Size, background: Rgb565) -> Panel {
        let mut panel_labels = Vec::with_capacity(7);

        let header_label = Label::new()
            .text("Indoor")
            .font(FONT_LUBS12)
            .font_color(Rgb565::CSS_DARK_RED)
            .backgound(background)
            .position(top_left + Point::new(160, 16));
        panel_labels.push(header_label);

        let mut title_label = Label::new()
            .text("Pressure")
            .position(top_left + Point::new(55, 44));
        panel_labels.push(title_label.clone());

        let mut value_label = Label::new()
            .text("--")
            .font_color(Rgb565::BLUE)
            .backgound(background)
            .position(top_left + Point::new(55, 62));
        panel_labels.push(value_label.clone());

        title_label = title_label
            .text("Temperature")
            .position(top_left + Point::new(160, 36));
        panel_labels.push(title_label.clone());

        value_label = value_label
            .text("--")
            .position(top_left + Point::new(160, 61));
        panel_labels.push(value_label.clone());

        title_label = title_label
            .text("Humidity")
            .position(top_left + Point::new(265, 44));
        panel_labels.push(title_label);

        value_label = value_label
            .text("--")
            .position(top_left + Point::new(265, 62));
        panel_labels.push(value_label);

        Panel::new(top_left, size, background, panel_labels)
    }
}

pub enum OutdoorValueId {
    Humidity = 2,
    Temperature = 4,
    DewPoint = 6,
    HeatIndex = 8,
    WindChill = 10,
}
struct OutdoorPanel {}

/**
 * Build outdoor panel
 *
 *           Panel Labels Vector Layout
 *        Element             Element Name
 *  --------------------------------------------------
 *          0           Header
 *          1           Humidity title
 *          2           Humidity value
 *          3           Temperature title
 *          4           Temperature  value
 *          5           Dew Point title
 *          6           Dew Point value
 *          7           Heat Index title
 *          8           Heat Index value
 *          9           Wind Chill title
 *          10          Wind Chill value
 */

impl OutdoorPanel {
    pub fn build(top_left: Point, size: Size, background: Rgb565) -> Panel {
        let mut panel_labels = Vec::with_capacity(11);

        let header_label = Label::new()
            .text("Outdoor")
            .font(FONT_LUBS12)
            .font_color(Rgb565::CSS_DARK_RED)
            .backgound(background)
            .position(top_left + Point::new(160, 14));
        panel_labels.push(header_label);

        let mut title_label = Label::new()
            .text("Humidity")
            .position(top_left + Point::new(55, 44));

        panel_labels.push(title_label.clone());

        let mut value_label = Label::new()
            .text("--")
            .font_color(Rgb565::BLUE)
            .backgound(background)
            .position(top_left + Point::new(55, 60));
        panel_labels.push(value_label.clone());

        title_label = title_label
            .text("Temperature")
            .position(top_left + Point::new(160, 36));
        panel_labels.push(title_label.clone());

        let temp_label = Label::new()
            .text("--")
            .font(FONT_LUBS24)
            .font_color(Rgb565::BLUE)
            .backgound(background)
            .position(top_left + Point::new(160, 61));
        panel_labels.push(temp_label);

        title_label = title_label
            .text("Dew Point")
            .position(top_left + Point::new(265, 44));
        panel_labels.push(title_label.clone());

        value_label = value_label
            .text("--")
            .position(top_left + Point::new(265, 60));
        panel_labels.push(value_label.clone());

        title_label = title_label
            .text("Heat Index")
            .position(top_left + Point::new(55, 84));
        panel_labels.push(title_label.clone());

        value_label = value_label
            .text("--")
            .position(top_left + Point::new(55, 98));
        panel_labels.push(value_label.clone());

        title_label = title_label
            .text("Wind Chill")
            .position(top_left + Point::new(265, 84));
        panel_labels.push(title_label);

        value_label = value_label
            .text("--")
            .position(top_left + Point::new(265, 98));
        panel_labels.push(value_label);

        Panel::new(top_left, size, background, panel_labels)
    }
}

pub enum TimeDateValueId {
    Time,
    Date,
}

struct TimeDatePanel {}

impl TimeDatePanel {
    pub fn build(top_left: Point, size: Size, background: Rgb565) -> Panel {
        let mut panel_labels = Vec::with_capacity(2);

        let mut value_label = Label::new()
            .text("--")
            .font(FONT_LUBS24)
            .font_color(Rgb565::YELLOW)
            .backgound(background)
            .position(top_left + Point::new(160, 81));
        panel_labels.push(value_label.clone());

        value_label = value_label
            .font(FONT_LUBS12)
            .text("--")
            .position(top_left + Point::new(160, 120));
        panel_labels.push(value_label);

        Panel::new(top_left, size, background, panel_labels)
    }
}

pub enum NavigationButtonId {
    Previous,
    Set,
    Next,
}

struct NavPanel {}

impl NavPanel {
    pub fn build(
        top_left: Point,
        size: Size,
        background: Rgb565,
        show_set_button: bool,
    ) -> NavigationPanel {
        let mut panel_buttons = Vec::with_capacity(3);

        panel_buttons.push(RoundedButton::new().text("PREV").btn(
            RoundedRectangle::with_equal_corners(
                Rectangle::new(top_left + Point::new(43, 8), Size::new(53, 25)),
                Size::new(5, 5),
            ),
        ));

        panel_buttons.push(
            RoundedButton::new()
                .text("SET")
                .btn(RoundedRectangle::with_equal_corners(
                    Rectangle::new(top_left + Point::new(129, 8), Size::new(53, 25)),
                    Size::new(5, 5),
                ))
                .is_visible(show_set_button),
        );

        panel_buttons.push(RoundedButton::new().text("NEXT").btn(
            RoundedRectangle::with_equal_corners(
                Rectangle::new(top_left + Point::new(222, 8), Size::new(53, 25)),
                Size::new(5, 5),
            ),
        ));

        NavigationPanel::new(top_left, size, background, panel_buttons)
    }
}
