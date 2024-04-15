pub enum Response {
    TurnOn,
    TurnOff,
}

impl Into<&str> for Response {
    fn into(self) -> &'static str {
        match self {
            Self::TurnOn => "ON",
            Self::TurnOff => "OFF",
        }
    }
}
