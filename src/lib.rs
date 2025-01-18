struct Mode;

impl rofi_mode::Mode<'_> for Mode {
    const NAME: &'static str = "todo\0";
    fn init(_api: rofi_mode::Api<'_>) -> Result<Self, ()> {
        Ok(Self)
    }
    fn entries(&mut self) -> usize { 0 }
    fn entry_content(&self, _line: usize) -> rofi_mode::String { unreachable!() }
    fn react(
        &mut self,
        _event: rofi_mode::Event,
        _input: &mut rofi_mode::String,
    ) -> rofi_mode::Action {
        rofi_mode::Action::Exit
    }
    fn matches(&self, _line: usize, _matcher: rofi_mode::Matcher<'_>) -> bool {
        unreachable!()
    }
}

rofi_mode::export_mode!(Mode);
