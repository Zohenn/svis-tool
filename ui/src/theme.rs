use catppuccin::{Colour, Flavour, FlavourColours};
use ratatui::style::Color;

const fn convert(color: Colour) -> Color {
    Color::Rgb(color.0, color.1, color.2)
}

const DEFAULT_FLAVOR: Flavour = Flavour::Mocha;
const DEFAULT_COLORS: FlavourColours = DEFAULT_FLAVOR.colours();

pub const TEXT: Color = convert(DEFAULT_COLORS.text);
pub const BACKGROUND: Color = convert(DEFAULT_COLORS.base);
pub const HIGHLIGHT: Color = convert(DEFAULT_COLORS.teal);
pub const HIGHLIGHT2: Color = convert(DEFAULT_COLORS.green);
pub const ERROR: Color = convert(DEFAULT_COLORS.red);
pub const FOCUS: Color = convert(DEFAULT_COLORS.yellow);
