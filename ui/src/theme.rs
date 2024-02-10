use catppuccin::{Colour, Flavour, FlavourColours};
use ratatui::style::Color;

const fn convert(color: Colour) -> Color {
    Color::Rgb(color.0, color.1, color.2)
}

const DEFAULT_FLAVOR: Flavour = Flavour::Mocha;
const DEFAULT_COLORS: FlavourColours = DEFAULT_FLAVOR.colours();

pub const BACKGROUND: Color = convert(DEFAULT_COLORS.base);
