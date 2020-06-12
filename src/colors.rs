pub const BLACK: Color = Color([0.0, 0.0, 0.0, 1.0]);
pub const WHITE: Color = Color([1.0, 1.0, 1.0, 1.0]);
pub const RED: Color = Color([1.0, 0.0, 0.0, 1.0]);
pub const GREEN: Color = Color([0.0, 1.0, 0.0, 1.0]);
pub const BLUE: Color = Color([0.0, 0.0, 1.0, 1.0]);

pub fn color_from_name<S>(color_name: S) -> Option<Color>
where
    S: AsRef<str>,
{
    match color_name.as_ref() {
        "black" => Some(BLACK),
        "white" => Some(WHITE),
        "red" => Some(RED),
        "green" => Some(GREEN),
        "blue" => Some(BLUE),
        _ => None,
    }
}

#[derive(Clone, Copy)]
pub struct Color([f32; 4]);

impl Color {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Color([red, green, blue, alpha])
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        self.0
    }
}
